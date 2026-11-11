/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    /// todo
    Red,
    /// todo
    Blue,
}
/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitKind {
    /// todo
    Basic,
}

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceError {
    /// todo
    OutOfBounds,
    /// todo
    Occupied,
    /// todo
    Unoccupied,
    /// todo
    OverBudget,
}

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    /// todo
    pub max_health: u32,
    /// todo
    pub cost: u32,
}

/// todo
pub struct Setup {
    board: Board,
    red_budget: u32,
    blue_budget: u32,
}

impl Setup {
    pub fn new(width: usize, height: usize, budget: u32) -> Setup {
        Setup {
            board: Board::new(width, height),
            red_budget: budget,
            blue_budget: budget,
        }
    }

    pub fn place(&mut self, row: usize, col: usize, kind: UnitKind) -> Result<(), PlaceError> {
        if self.board.index(row, col).is_none() {
            return Err(PlaceError::OutOfBounds);
        }

        if self.board.get(row, col).is_some() {
            return Err(PlaceError::Occupied);
        }

        let team = if col >= self.board.width / 2 {
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

/// todo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unit {
    /// todo
    pub team: Team,
    /// todo
    pub kind: UnitKind,
    /// todo
    pub health: u32,
}

/// todo
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Option<Unit>>,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        Board {
            width,
            height,
            cells: vec![None; width * height],
        }
    }

    fn index(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.height && col < self.width {
            return Some(row * self.width + col);
        }
        None
    }

    pub fn get(&self, row: usize, col: usize) -> Option<Unit> {
        let i = self.index(row, col)?;
        self.cells[i]
    }

    pub fn set(&mut self, row: usize, col: usize, unit: Option<Unit>) -> bool {
        let Some(i) = self.index(row, col) else {
            return false;
        };
        self.cells[i] = unit;
        true
    }
}

impl UnitKind {
    /// todo
    pub fn stats(self) -> Stats {
        match self {
            UnitKind::Basic => Stats {
                max_health: 4,
                cost: 1,
            },
        }
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

    fn red() -> Unit {
        Unit {
            team: Team::Red,
            kind: UnitKind::Basic,
            health: UnitKind::Basic.stats().max_health,
        }
    }

    #[test]
    fn basic_costs_one() {
        assert_eq!(UnitKind::Basic.stats().cost, 1);
    }

    #[test]
    fn new_board_is_empty() {
        let board = Board::new(3, 3);
        assert_eq!(board.get(1, 1), None);
    }

    #[test]
    fn set_then_get_returns_unit() {
        let mut board = Board::new(3, 3);
        assert!(board.set(0, 0, Some(red())));
        assert_eq!(board.get(0, 0), Some(red()));
    }

    #[test]
    fn set_touches_only_its_own_cell() {
        let mut board = Board::new(3, 4);
        let target_row = 1;
        let target_col = 2;
        assert!(board.set(target_row, target_col, Some(red())));
        for row in 0..board.height {
            for col in 0..board.width {
                if row == target_row && col == target_col {
                    assert_eq!(board.get(row, col), Some(red()));
                } else {
                    assert_eq!(board.get(row, col), None);
                }
            }
        }
    }

    #[test]
    fn set_none_clears_cell() {
        let mut board = Board::new(3, 3);
        assert!(board.set(1, 2, Some(red())));
        assert_eq!(board.get(1, 2), Some(red()));
        assert!(board.set(1, 2, None));
        assert_eq!(board.get(1, 2), None);
    }

    #[test]
    fn last_cell_is_valid() {
        let mut board = Board::new(3, 3);
        assert!(board.set(2, 2, Some(red())));
        assert_eq!(board.get(2, 2), Some(red()));
    }

    #[test]
    fn out_of_bounds_get_is_none() {
        let board = Board::new(3, 3);
        assert_eq!(board.get(3, 0), None);
        assert_eq!(board.get(0, 3), None);
    }

    #[test]
    fn out_of_bounds_set_fails() {
        let mut board = Board::new(3, 4);
        assert!(!board.set(4, 0, Some(red())));
        assert!(!board.set(0, 3, Some(red())));
        for row in 0..board.height {
            for col in 0..board.width {
                assert_eq!(board.get(row, col), None);
            }
        }
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
        s.place(0, s.board.width - 1, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, s.board.width - 1), Team::Red);
    }

    #[test]
    fn place_on_zone_boundary_is_red() {
        let mut s = setup();
        s.place(0, s.board.width / 2, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, s.board.width / 2), Team::Red);
    }

    #[test]
    fn place_out_of_bounds_fails() {
        let mut s = setup();
        assert_eq!(
            s.place(0, s.board.width, UnitKind::Basic),
            Err(PlaceError::OutOfBounds)
        );
        assert_eq!(
            s.place(s.board.height, 0, UnitKind::Basic),
            Err(PlaceError::OutOfBounds)
        );
    }

    #[test]
    fn place_on_occupied_cell_fails() {
        let mut s = setup();
        s.place(0, s.board.width / 2, UnitKind::Basic).unwrap();
        assert_eq!(team_at(&s, 0, s.board.width / 2), Team::Red);
        assert_eq!(
            s.place(0, s.board.width / 2, UnitKind::Basic),
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
        assert_eq!(s.remove(s.board.height, 0), Err(PlaceError::OutOfBounds));
        assert_eq!(s.remove(0, s.board.width), Err(PlaceError::OutOfBounds));
        assert_eq!(s.blue_budget, 4);
    }
}
