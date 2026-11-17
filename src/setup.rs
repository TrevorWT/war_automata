use crate::board::Board;
use crate::unit::{Team, Unit, UnitKind};

/// Why a placement or removal was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceError {
    /// The cell is off the board.
    OutOfBounds,
    /// The cell already holds a unit.
    Occupied,
    /// The cell holds no unit to remove.
    Unoccupied,
    /// The team cannot afford the unit.
    OverBudget,
}

/// The placement phase: a board plus each team's remaining budget.
/// The right half of the board belongs to Red, the left half to Blue.
pub struct Setup {
    board: Board,
    /// Points each team started with. Spent points are `limit - remaining`.
    limit: u32,
    red_budget: u32,
    blue_budget: u32,
}

impl Setup {
    /// An empty setup where each team starts with `budget` points.
    pub fn new(width: usize, height: usize, budget: u32) -> Setup {
        Setup {
            board: Board::new(width, height),
            limit: budget,
            red_budget: budget,
            blue_budget: budget,
        }
    }

    /// The board as placed so far.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Total points each team gets to spend.
    pub fn limit(&self) -> u32 {
        self.limit
    }

    /// The lowest limit allowed right now: the most either team has already spent.
    pub fn min_limit(&self) -> u32 {
        (self.limit - self.red_budget).max(self.limit - self.blue_budget)
    }

    /// Sets each team's total budget, keeping what they have already spent.
    /// A limit below `min_limit` is raised to it. Returns the limit now in effect.
    pub fn set_limit(&mut self, limit: u32) -> u32 {
        let new = limit.max(self.min_limit());
        let red_spent = self.limit - self.red_budget;
        let blue_spent = self.limit - self.blue_budget;
        self.red_budget = new - red_spent;
        self.blue_budget = new - blue_spent;
        self.limit = new;
        new
    }

    /// Points `team` still has to spend.
    pub fn budget(&self, team: Team) -> u32 {
        match team {
            Team::Red => self.red_budget,
            Team::Blue => self.blue_budget,
        }
    }

    /// Hands over the finished board to start a battle.
    pub fn into_board(self) -> Board {
        self.board
    }

    /// Places a `kind` unit for the team that owns the column and spends its cost.
    pub fn place(&mut self, row: usize, col: usize, kind: UnitKind) -> Result<(), PlaceError> {
        if self.board.index(row, col).is_none() {
            return Err(PlaceError::OutOfBounds);
        }

        if self.board.get(row, col).is_some() {
            return Err(PlaceError::Occupied);
        }

        let team = if col >= self.board.width() / 2 {
            Team::Red
        } else {
            Team::Blue
        };

        let budget = match team {
            Team::Red => &mut self.red_budget,
            Team::Blue => &mut self.blue_budget,
        };

        let stats = kind.stats();
        if *budget < stats.cost {
            return Err(PlaceError::OverBudget);
        }
        *budget -= stats.cost;

        let unit = Unit {
            team,
            kind,
            health: stats.max_health,
        };
        self.board.set(row, col, Some(unit));

        Ok(())
    }

    /// Removes the unit at `(row, col)` and refunds its cost.
    pub fn remove(&mut self, row: usize, col: usize) -> Result<(), PlaceError> {
        if self.board.index(row, col).is_none() {
            return Err(PlaceError::OutOfBounds);
        }

        let unit = self.board.get(row, col).ok_or(PlaceError::Unoccupied)?;

        let budget = match unit.team {
            Team::Red => &mut self.red_budget,
            Team::Blue => &mut self.blue_budget,
        };

        *budget += unit.kind.stats().cost;

        self.board.set(row, col, None);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Setup {
        Setup::new(4, 4, 10)
    }

    fn team_at(setup: &Setup, row: usize, col: usize) -> Team {
        setup.board.get(row, col).unwrap().team
    }

    #[test]
    fn budget_drops_on_place_and_returns_on_remove() {
        let mut s = setup();
        s.place(0, 0, UnitKind::Healer).unwrap();
        assert_eq!(s.budget(Team::Blue), 10 - UnitKind::Healer.stats().cost);
        assert_eq!(s.budget(Team::Red), 10);
        s.remove(0, 0).unwrap();
        assert_eq!(s.budget(Team::Blue), 10);
    }

    #[test]
    fn raising_the_limit_adds_to_remaining_budget() {
        let mut s = setup();
        s.place(0, 0, UnitKind::Basic).unwrap();
        assert_eq!(s.set_limit(20), 20);
        assert_eq!(s.limit(), 20);
        assert_eq!(s.budget(Team::Blue), 19);
        assert_eq!(s.budget(Team::Red), 20);
    }

    #[test]
    fn lowering_the_limit_keeps_spent_points() {
        let mut s = setup();
        s.place(0, 0, UnitKind::Commander).unwrap();
        assert_eq!(s.set_limit(7), 7);
        assert_eq!(s.budget(Team::Blue), 7 - UnitKind::Commander.stats().cost);
        assert!(s.board().get(0, 0).is_some());
    }

    #[test]
    fn limit_cannot_drop_below_what_is_spent() {
        let mut s = setup();
        let cost = UnitKind::Commander.stats().cost;
        s.place(0, 0, UnitKind::Commander).unwrap();
        assert_eq!(s.min_limit(), cost);
        assert_eq!(s.set_limit(0), cost);
        assert_eq!(s.budget(Team::Blue), 0);
        assert_eq!(s.budget(Team::Red), cost);
    }

    #[test]
    fn board_getter_sees_placed_units() {
        let mut s = setup();
        s.place(1, 1, UnitKind::Basic).unwrap();
        assert!(s.board().get(1, 1).is_some());
    }

    #[test]
    fn place_left_half_is_blue() {
        let mut s = setup();
        s.place(0, 0, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, 0), Team::Blue);
    }

    #[test]
    fn place_right_half_is_red() {
        let mut s = setup();
        s.place(0, s.board.width() - 1, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, s.board.width() - 1), Team::Red);
    }

    #[test]
    fn place_on_zone_boundary_is_red() {
        let mut s = setup();
        s.place(0, s.board.width() / 2, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, s.board.width() / 2), Team::Red);
    }

    #[test]
    fn place_out_of_bounds_fails() {
        let mut s = setup();
        assert_eq!(
            s.place(0, s.board.width(), UnitKind::Basic),
            Err(PlaceError::OutOfBounds)
        );
        assert_eq!(
            s.place(s.board.height(), 0, UnitKind::Basic),
            Err(PlaceError::OutOfBounds)
        );
    }

    #[test]
    fn place_on_occupied_cell_fails() {
        let mut s = setup();
        s.place(0, s.board.width() / 2, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, s.board.width() / 2), Team::Red);
        assert_eq!(
            s.place(0, s.board.width() / 2, UnitKind::Basic),
            Err(PlaceError::Occupied)
        );
    }

    #[test]
    fn place_over_budget_fails() {
        let mut s = Setup::new(4, 4, 0);
        assert_eq!(s.place(0, 0, UnitKind::Basic), Err(PlaceError::OverBudget));
        assert_eq!(s.place(0, 1, UnitKind::Basic), Err(PlaceError::OverBudget));
    }

    #[test]
    fn place_spends_budget() {
        let mut s = Setup::new(12, 12, 4);
        for i in 0..4 {
            s.place(i as usize, 0, UnitKind::Basic).unwrap();
            assert_eq!(s.blue_budget, 4 - (i + 1));
        }
        assert_eq!(s.place(0, 1, UnitKind::Basic), Err(PlaceError::OverBudget));
    }

    #[test]
    fn place_failure_does_not_spend_budget() {
        let mut s = Setup::new(4, 4, 4);
        s.place(0, 0, UnitKind::Basic).unwrap();
        assert_eq!(s.blue_budget, 3);
        assert_eq!(s.place(0, 0, UnitKind::Basic), Err(PlaceError::Occupied));
        assert_eq!(s.blue_budget, 3);
    }

    #[test]
    fn teams_have_separate_budgets() {
        let mut s = Setup::new(4, 4, 4);
        assert_eq!(s.place(0, 0, UnitKind::Basic), Ok(()));
        assert_eq!(s.blue_budget, 3);
        assert_eq!(s.red_budget, 4);
        assert_eq!(s.place(3, 3, UnitKind::Basic), Ok(()));
        assert_eq!(s.place(3, 2, UnitKind::Basic), Ok(()));
        assert_eq!(s.red_budget, 2);
        assert_eq!(s.blue_budget, 3);
    }

    #[test]
    fn remove_clears_cell() {
        let mut s = Setup::new(4, 4, 4);
        s.place(0, 0, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, 0), Team::Blue);
        s.remove(0, 0).unwrap();
        assert_eq!(s.board.get(0, 0), None);
    }

    #[test]
    fn remove_refunds_budget() {
        let mut s = Setup::new(4, 4, 4);
        s.place(0, 0, UnitKind::Basic).unwrap();
        assert_eq!(s.blue_budget, 3);
        s.remove(0, 0).unwrap();
        assert_eq!(s.blue_budget, 4);
    }

    #[test]
    fn remove_refunds_correct_team() {
        let mut s = Setup::new(4, 4, 4);
        s.place(0, 0, UnitKind::Basic).unwrap();
        assert_eq!(s.blue_budget, 3);
        s.remove(0, 0).unwrap();
        assert_eq!(s.blue_budget, 4);

        s.place(3, 3, UnitKind::Basic).unwrap();
        assert_eq!(s.red_budget, 3);
        s.remove(3, 3).unwrap();
        assert_eq!(s.red_budget, 4);
    }

    #[test]
    fn remove_empty_cell_fails() {
        let mut s = Setup::new(4, 4, 4);
        assert_eq!(s.remove(0, 0), Err(PlaceError::Unoccupied));
        assert_eq!(s.blue_budget, 4);
    }

    #[test]
    fn remove_out_of_bounds_fails() {
        let mut s = Setup::new(4, 4, 4);
        assert_eq!(s.remove(s.board.height(), 0), Err(PlaceError::OutOfBounds));
        assert_eq!(s.remove(0, s.board.width()), Err(PlaceError::OutOfBounds));
        assert_eq!(s.blue_budget, 4);
    }
}
