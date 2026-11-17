use crate::board::Board;
use crate::rules::{Effect, apply_conversion, apply_damage, distance, nearest_enemy, take_turn};
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
    effects: Vec<Effect>,
    rng: Rng,
    /// The team that moves first in round 1. Rounds then alternate.
    first_mover: Team,
}

impl Battle {
    /// Starts a battle on `board` at round 0 with a fixed seed, so it always
    /// plays out the same way.
    pub fn new(board: Board) -> Battle {
        Battle::with_seed(board, 0)
    }

    /// Starts a battle whose move-order ties are broken by `seed`. The same seed
    /// on the same board always gives the same battle.
    pub fn with_seed(board: Board, seed: u64) -> Battle {
        Battle {
            board,
            round: 0,
            effects: Vec::new(),
            rng: Rng(seed),
            first_mover: Team::Red,
        }
    }

    /// Sets which team moves first in round 1 (Red by default). Later rounds
    /// alternate from there.
    pub fn with_first_mover(mut self, team: Team) -> Battle {
        self.first_mover = team;
        self
    }

    /// What happened in the most recent round (damage, healing, deaths,
    /// conversions), for display. Empty before the first round.
    pub fn effects(&self) -> &[Effect] {
        &self.effects
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

    /// Plays one round: both teams move, then damage hits everyone at once.
    /// The first mover alternates: the starting team moves first on rounds
    /// 1, 3, 5... and the other team on rounds 2, 4, 6... Does nothing once the
    /// battle is over.
    pub fn round(&mut self) {
        if self.outcome() != Outcome::Ongoing {
            return;
        }
        let first = if self.round % 2 == 0 {
            self.first_mover
        } else {
            self.first_mover.opponent()
        };
        let second = first.opponent();
        self.move_team(first);
        self.move_team(second);
        self.effects = apply_damage(&mut self.board);
        self.effects.extend(apply_conversion(&mut self.board));
        self.round += 1;
    }

    /// Gives each unit of `team` one turn, front line first: units closest to
    /// their nearest enemy move first, so they clear room for those behind.
    /// Units at the same distance are ordered randomly (from the battle's seed).
    /// The order is fixed before anyone moves, so no unit moves twice.
    fn move_team(&mut self, team: Team) {
        let mut order = Vec::new();
        for row in 0..self.board.height() {
            for col in 0..self.board.width() {
                if self.board.get(row, col).is_none_or(|u| u.team != team) {
                    continue;
                }
                // Only the distance is needed here, which ties cannot change.
                let Some(target) = nearest_enemy(&self.board, (row, col), team, &mut |_| 0) else {
                    continue;
                };
                order.push((distance((row, col), target), self.rng.next(), (row, col)));
            }
        }
        order.sort();
        // Exact ties (equal cells for a step, equal enemies) are settled randomly.
        let rng = &mut self.rng;
        let mut pick = |n: usize| rng.below(n);
        for (_, _, pos) in order {
            take_turn(&mut self.board, pos, &mut pick);
        }
    }
}

/// Small seedable random number generator (splitmix64), so battles can be
/// varied from run to run and still be repeated exactly from a seed.
struct Rng(u64);

impl Rng {
    /// A random index in `0..n`. `n` must be above 0.
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::EffectKind;
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
    fn effects_report_the_latest_round() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 2, Some(unit(Team::Blue)));
        let mut battle = Battle::new(board);
        assert!(battle.effects().is_empty());
        battle.round();
        // Red steps to the middle and touches Blue: both take damage.
        assert_eq!(battle.effects().len(), 2);
        assert!(
            battle
                .effects()
                .iter()
                .all(|e| e.kind == EffectKind::Damage)
        );
    }

    #[test]
    fn front_line_moves_first_for_both_teams() {
        // Blue advances right: its front unit is the one at column 1, and it must
        // move before the unit behind it or the rear unit is stuck.
        let mut board = Board::new(6, 1);
        board.set(0, 0, Some(unit(Team::Blue)));
        board.set(0, 1, Some(unit(Team::Blue)));
        board.set(0, 5, Some(unit(Team::Red)));
        let mut battle = Battle::new(board);
        battle.round();
        assert_eq!(battle.board().get(0, 1), Some(unit(Team::Blue)));
        assert_eq!(battle.board().get(0, 2), Some(unit(Team::Blue)));
        assert_eq!(battle.board().get(0, 0), None);
    }

    /// Every cell of the board, to compare whole battles.
    fn snapshot(battle: &Battle) -> Vec<Option<Unit>> {
        let board = battle.board();
        let mut cells = Vec::new();
        for row in 0..board.height() {
            for col in 0..board.width() {
                cells.push(board.get(row, col));
            }
        }
        cells
    }

    fn crowded_board() -> Board {
        let mut board = Board::new(10, 5);
        for row in 0..5 {
            for col in 0..3 {
                board.set(row, col, Some(unit(Team::Blue)));
                board.set(row, 9 - col, Some(unit(Team::Red)));
            }
        }
        board
    }

    fn play(seed: u64) -> Vec<Option<Unit>> {
        let mut battle = Battle::with_seed(crowded_board(), seed);
        for _ in 0..6 {
            battle.round();
        }
        snapshot(&battle)
    }

    #[test]
    fn same_seed_gives_the_same_battle() {
        assert_eq!(play(7), play(7));
    }

    #[test]
    fn tied_units_race_for_a_cell_in_a_seed_dependent_order() {
        // Two Red units, both distance 2 from the lone Blue unit, both want the
        // single best cell (1, 1). Whichever moves first gets it.
        let play_one_round = |seed| {
            let mut board = Board::new(3, 3);
            board.set(1, 0, Some(unit(Team::Blue)));
            board.set(0, 2, Some(unit(Team::Red)));
            board.set(2, 2, Some(unit(Team::Red)));
            let mut battle = Battle::with_seed(board, seed);
            battle.round();
            snapshot(&battle)
        };
        let outcomes: Vec<_> = (0..20).map(play_one_round).collect();
        assert!(outcomes.iter().any(|o| *o != outcomes[0]));
    }

    #[test]
    fn first_mover_alternates_each_round() {
        let contested = |rounds_played| {
            let mut board = Board::new(3, 1);
            board.set(0, 0, Some(unit(Team::Red)));
            board.set(0, 2, Some(unit(Team::Blue)));
            let mut battle = Battle::new(board);
            battle.round = rounds_played;
            battle.round();
            battle.board().get(0, 1).unwrap().team
        };
        // Round 1 (0 played): Red moves first and takes the middle cell.
        assert_eq!(contested(0), Team::Red);
        // Round 2 (1 played): Blue moves first.
        assert_eq!(contested(1), Team::Blue);
        // Round 3 (2 played): back to Red.
        assert_eq!(contested(2), Team::Red);
    }

    #[test]
    fn first_mover_can_start_with_blue() {
        let contested = |rounds_played| {
            let mut board = Board::new(3, 1);
            board.set(0, 0, Some(unit(Team::Red)));
            board.set(0, 2, Some(unit(Team::Blue)));
            let mut battle = Battle::new(board).with_first_mover(Team::Blue);
            battle.round = rounds_played;
            battle.round();
            battle.board().get(0, 1).unwrap().team
        };
        // Round 1: Blue first. Round 2: Red first. Round 3: Blue again.
        assert_eq!(contested(0), Team::Blue);
        assert_eq!(contested(1), Team::Red);
        assert_eq!(contested(2), Team::Blue);
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
