use crate::board::Board;
use crate::rules::{apply_conversion, apply_damage, take_turn};
use crate::unit::Team;

/// Rounds after which an unfinished battle is called a draw.
pub const ROUND_LIMIT: u32 = 500;

/// How a battle stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Both teams still have units and the round limit is not reached.
    Ongoing,
    /// The other team has no units left.
    Won(Team),
    /// Both teams wiped out together, or the round limit was reached.
    Draw,
}

/// A running battle: the board plus the number of rounds played.
pub struct Battle {
    board: Board,
    round: u32,
}

impl Battle {
    /// Starts a battle on `board` at round 0.
    pub fn new(board: Board) -> Battle {
        Battle { board, round: 0 }
    }

    /// The current board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Number of rounds played so far.
    pub fn round_count(&self) -> u32 {
        self.round
    }

    /// Current outcome, judged from living units and the round limit.
    pub fn outcome(&self) -> Outcome {
        let mut red = 0;
        let mut blue = 0;
        for row in 0..self.board.height() {
            for col in 0..self.board.width() {
                match self.board.get(row, col).map(|u| u.team) {
                    Some(Team::Red) => red += 1,
                    Some(Team::Blue) => blue += 1,
                    None => {}
                }
            }
        }
        match (red, blue) {
            (0, 0) => Outcome::Draw,
            (0, _) => Outcome::Won(Team::Blue),
            (_, 0) => Outcome::Won(Team::Red),
            _ if self.round >= ROUND_LIMIT => Outcome::Draw,
            _ => Outcome::Ongoing,
        }
    }

    /// Plays one round: Red moves, then Blue moves, then damage hits everyone at once.
    /// Does nothing once the battle is over.
    pub fn round(&mut self) {
        if self.outcome() != Outcome::Ongoing {
            return;
        }
        self.move_team(Team::Red);
        self.move_team(Team::Blue);
        apply_damage(&mut self.board);
        apply_conversion(&mut self.board);
        self.round += 1;
    }

    /// Gives each unit of `team` one turn, scanning row-major.
    /// Positions are collected first so a unit that moves is not scanned again.
    fn move_team(&mut self, team: Team) {
        let mut positions = Vec::new();
        for row in 0..self.board.height() {
            for col in 0..self.board.width() {
                if self.board.get(row, col).is_some_and(|u| u.team == team) {
                    positions.push((row, col));
                }
            }
        }
        for pos in positions {
            take_turn(&mut self.board, pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::{Unit, UnitKind};

    fn unit(team: Team) -> Unit {
        Unit {
            team,
            kind: UnitKind::Basic,
            health: UnitKind::Basic.stats().max_health,
        }
    }

    fn two_unit_battle(width: usize) -> Battle {
        let mut board = Board::new(width, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, width - 1, Some(unit(Team::Blue)));
        Battle::new(board)
    }

    #[test]
    fn round_counts_up() {
        let mut battle = two_unit_battle(20);
        assert_eq!(battle.round_count(), 0);
        battle.round();
        battle.round();
        assert_eq!(battle.round_count(), 2);
    }

    #[test]
    fn outcome_ongoing_with_both_teams() {
        assert_eq!(two_unit_battle(20).outcome(), Outcome::Ongoing);
    }

    #[test]
    fn outcome_win_when_one_team_remains() {
        let mut board = Board::new(3, 3);
        board.set(0, 0, Some(unit(Team::Red)));
        assert_eq!(Battle::new(board).outcome(), Outcome::Won(Team::Red));
        let mut board = Board::new(3, 3);
        board.set(0, 0, Some(unit(Team::Blue)));
        assert_eq!(Battle::new(board).outcome(), Outcome::Won(Team::Blue));
    }

    #[test]
    fn outcome_draw_when_board_empty() {
        assert_eq!(Battle::new(Board::new(3, 3)).outcome(), Outcome::Draw);
    }

    #[test]
    fn outcome_draw_at_round_limit() {
        let mut battle = two_unit_battle(20);
        battle.round = ROUND_LIMIT - 1;
        assert_eq!(battle.outcome(), Outcome::Ongoing);
        battle.round();
        assert_eq!(battle.round_count(), ROUND_LIMIT);
        assert_eq!(battle.outcome(), Outcome::Draw);
    }

    #[test]
    fn round_does_nothing_after_the_end() {
        let mut board = Board::new(3, 3);
        board.set(0, 0, Some(unit(Team::Red)));
        let mut battle = Battle::new(board);
        battle.round();
        assert_eq!(battle.round_count(), 0);
    }

    #[test]
    fn battle_ends_with_a_winner() {
        // Red has two units, Blue one: Blue dies first.
        let mut board = Board::new(3, 2);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(1, 0, Some(unit(Team::Red)));
        board.set(0, 2, Some(unit(Team::Blue)));
        let mut battle = Battle::new(board);
        for _ in 0..ROUND_LIMIT {
            battle.round();
        }
        assert_eq!(battle.outcome(), Outcome::Won(Team::Red));
    }

    #[test]
    fn each_unit_moves_once_per_round() {
        let mut board = Board::new(5, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 4, Some(unit(Team::Blue)));
        let mut battle = Battle::new(board);
        battle.round();
        assert_eq!(battle.board().get(0, 1), Some(unit(Team::Red)));
        assert_eq!(battle.board().get(0, 3), Some(unit(Team::Blue)));
        assert_eq!(battle.board().get(0, 2), None);
    }

    #[test]
    fn red_moves_before_blue_then_damage_applies() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 2, Some(unit(Team::Blue)));
        let mut battle = Battle::new(board);
        battle.round();
        // Red takes the contested middle cell first; Blue cannot move in.
        let red = battle.board().get(0, 1).unwrap();
        let blue = battle.board().get(0, 2).unwrap();
        assert_eq!((red.team, blue.team), (Team::Red, Team::Blue));
        // Touching after movement, so both take damage this round.
        assert_eq!(red.health, 3);
        assert_eq!(blue.health, 3);
    }

    #[test]
    fn priest_conversion_can_win_the_battle() {
        let mut board = Board::new(3, 1);
        board.set(
            0,
            0,
            Some(Unit {
                team: Team::Red,
                kind: UnitKind::Priest,
                health: UnitKind::Priest.stats().max_health,
            }),
        );
        board.set(0, 2, Some(unit(Team::Blue)));
        let mut battle = Battle::new(board);
        battle.round();
        assert_eq!(battle.board().get(0, 2).unwrap().team, Team::Red);
        assert_eq!(battle.outcome(), Outcome::Won(Team::Red));
    }

    #[test]
    fn distant_units_still_close_in() {
        let mut board = Board::new(10, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 9, Some(unit(Team::Blue)));
        let mut battle = Battle::new(board);
        battle.round();
        assert_eq!(battle.board().get(0, 1), Some(unit(Team::Red)));
        assert_eq!(battle.board().get(0, 8), Some(unit(Team::Blue)));
    }
}
