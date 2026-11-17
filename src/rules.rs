use crate::board::Board;
use crate::unit::{Action, Band, Team, Unit};

/// Squared straight-line distance. Used only to break Chebyshev ties: of two
/// equally near cells, the straighter line to the target has the smaller value,
/// so units head straight at their target instead of drifting toward one edge.
fn straight_sq(a: (usize, usize), b: (usize, usize)) -> usize {
    let row_gap = a.0.abs_diff(b.0);
    let col_gap = a.1.abs_diff(b.1);
    row_gap * row_gap + col_gap * col_gap
}

/// Chooses among cells that are still exactly tied. `pick(n)` returns an index
/// in `0..n`; the battle passes a seeded random choice, and tests pass fixed ones.
/// With one candidate there is nothing to choose, so `pick` is not called.
fn choose(ties: &[(usize, usize)], pick: &mut dyn FnMut(usize) -> usize) -> Option<(usize, usize)> {
    match ties.len() {
        0 => None,
        1 => Some(ties[0]),
        n => Some(ties[pick(n)]),
    }
}

/// Position of the enemy of `team` closest to `pos` (Chebyshev distance).
/// Ties go to the straighter line; enemies still tied after that are chosen
/// between with `pick`. `None` if no enemy exists.
pub fn nearest_enemy(
    board: &Board,
    pos: (usize, usize),
    team: Team,
    pick: &mut dyn FnMut(usize) -> usize,
) -> Option<(usize, usize)> {
    let mut best_key: Option<(usize, usize)> = None;
    let mut ties = Vec::new();
    // Scanning row-major keeps `ties` in a fixed order, so a seed is repeatable.
    for row in 0..board.height() {
        for col in 0..board.width() {
            let Some(unit) = board.get(row, col) else {
                continue;
            };
            if unit.team == team {
                continue;
            }
            let key = (distance(pos, (row, col)), straight_sq(pos, (row, col)));
            match best_key {
                Some(best) if key > best => {}
                Some(best) if key == best => ties.push((row, col)),
                _ => {
                    best_key = Some(key);
                    ties.clear();
                    ties.push((row, col));
                }
            }
        }
    }
    choose(&ties, pick)
}

/// Empty neighbor cell of `pos` that `action` would move to, if any.
/// `Advance` needs a cell strictly closer to `target`; `Sidestep` needs one at
/// the same distance. Among valid cells the nearest to `target` wins, then the
/// straighter line; cells still tied after that are chosen between with `pick`.
/// Other actions never move yet.
pub fn pick_cell(
    board: &Board,
    pos: (usize, usize),
    target: (usize, usize),
    action: Action,
    pick: &mut dyn FnMut(usize) -> usize,
) -> Option<(usize, usize)> {
    let current = distance(pos, target);
    let mut best_key: Option<(usize, usize)> = None;
    let mut ties = Vec::new();
    // neighbors() is row-major, so `ties` is in a fixed order.
    for cell in board.neighbors(pos.0, pos.1) {
        if board.get(cell.0, cell.1).is_some() {
            continue;
        }
        let d = distance(cell, target);
        let allowed = match action {
            Action::Advance => d < current,
            Action::Sidestep => d == current,
            _ => false,
        };
        if !allowed {
            continue;
        }
        let key = (d, straight_sq(cell, target));
        match best_key {
            Some(best) if key > best => {}
            Some(best) if key == best => ties.push(cell),
            _ => {
                best_key = Some(key);
                ties.clear();
                ties.push(cell);
            }
        }
    }
    choose(&ties, pick)
}

/// Walks `bands` in order and returns the cell the first usable one moves to.
/// A band is usable when `dist` is within its `max_dist`. If its action finds no
/// cell, the walk falls through to the next band. `Hold` always succeeds without
/// moving, so it ends the walk with `None`.
fn walk_bands(
    board: &Board,
    pos: (usize, usize),
    target: (usize, usize),
    bands: &[Band],
    pick: &mut dyn FnMut(usize) -> usize,
) -> Option<(usize, usize)> {
    let dist = distance(pos, target);
    for band in bands {
        if dist > band.max_dist as usize {
            continue;
        }
        if band.action == Action::Hold {
            return None;
        }
        if let Some(cell) = pick_cell(board, pos, target, band.action, pick) {
            return Some(cell);
        }
    }
    None
}

/// Runs one unit's move. Finds the nearest enemy, walks the unit's bands, and
/// moves it to the chosen cell. Returns the new position, or `None` if the cell
/// is empty, no enemy exists, or nothing moved. Exact ties are settled by `pick`.
pub fn take_turn(
    board: &mut Board,
    pos: (usize, usize),
    pick: &mut dyn FnMut(usize) -> usize,
) -> Option<(usize, usize)> {
    let unit = board.get(pos.0, pos.1)?;
    let target = nearest_enemy(board, pos, unit.team, pick)?;
    let dest = walk_bands(board, pos, target, unit.kind.stats().bands, pick)?;
    board.set(pos.0, pos.1, None);
    board.set(dest.0, dest.1, Some(unit));
    Some(dest)
}

/// Health after taking `damage` and receiving `heal`: net change is
/// `damage - heal`, computed in `i32` and clamped to `0..=max_health`.
fn next_health(health: u32, max_health: u32, damage: u32, heal: u32) -> u32 {
    let net = damage as i32 - heal as i32;
    (health as i32 - net).clamp(0, max_health as i32) as u32
}

/// Total `boost` from the allies of `unit` touching `pos`.
fn ally_boost(board: &Board, pos: (usize, usize), unit: Unit) -> u32 {
    board
        .neighbors(pos.0, pos.1)
        .into_iter()
        .filter_map(|(r, c)| board.get(r, c))
        .filter(|ally| ally.team == unit.team)
        .map(|ally| ally.kind.stats().boost)
        .sum()
}

/// Damage `unit` at `pos` deals this round: its own strength plus the `boost` of
/// each touching ally. Units with no strength (healers, priests) stay harmless.
fn effective_strength(board: &Board, pos: (usize, usize), unit: Unit) -> u32 {
    let base = unit.kind.stats().strength;
    if base == 0 {
        return 0;
    }
    base + ally_boost(board, pos, unit)
}

/// Health `unit` at `pos` restores to each touching ally this round: its own
/// `heal` plus the `boost` of each touching ally. Units with no `heal` (fighters,
/// priests) stay at 0.
fn effective_heal(board: &Board, pos: (usize, usize), unit: Unit) -> u32 {
    let base = unit.kind.stats().heal;
    if base == 0 {
        return 0;
    }
    base + ally_boost(board, pos, unit)
}

/// Whether the unit at `(row, col)` currently deals extra damage or healing
/// because a touching ally boosts it. `false` for an empty cell.
pub fn is_boosted(board: &Board, row: usize, col: usize) -> bool {
    board.get(row, col).is_some_and(|unit| {
        let stats = unit.kind.stats();
        effective_strength(board, (row, col), unit) > stats.strength
            || effective_heal(board, (row, col), unit) > stats.heal
    })
}

/// What happened to a cell this round, for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    /// The unit lost health.
    Damage,
    /// The unit gained health.
    Heal,
    /// The unit was removed at 0 health.
    Death,
    /// The unit was converted to the other team.
    Convert,
}

/// An [`EffectKind`] at a board cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Effect {
    /// Row of the affected cell.
    pub row: usize,
    /// Column of the affected cell.
    pub col: usize,
    /// What happened there.
    pub kind: EffectKind,
    /// The unit's team: for a death, the team that lost it; for a conversion,
    /// the team it joined.
    pub team: Team,
}

/// Every converting unit flips all of its touching enemies to its own team. A
/// flipped unit keeps its kind and health. Runs after damage, and all flips are
/// decided before any are applied, so two touching opposing priests swap sides.
/// If priests of both teams touch the same unit, the one later in row-major
/// order wins it.
/// Returns one `Convert` effect for each flipped cell.
pub fn apply_conversion(board: &mut Board) -> Vec<Effect> {
    let mut flips: Vec<(usize, usize, Unit)> = Vec::new();
    for row in 0..board.height() {
        for col in 0..board.width() {
            let Some(unit) = board.get(row, col) else {
                continue;
            };
            if !unit.kind.stats().converts {
                continue;
            }
            for (r, c) in board.neighbors(row, col) {
                let Some(enemy) = board.get(r, c) else {
                    continue;
                };
                if enemy.team == unit.team {
                    continue;
                }
                // A cell flipped twice keeps only the latest, so it is applied
                // and reported once.
                flips.retain(|&(fr, fc, _)| (fr, fc) != (r, c));
                flips.push((
                    r,
                    c,
                    Unit {
                        team: unit.team,
                        ..enemy
                    },
                ));
            }
        }
    }
    let mut effects = Vec::new();
    for (row, col, unit) in flips {
        board.set(row, col, Some(unit));
        effects.push(Effect {
            row,
            col,
            kind: EffectKind::Convert,
            team: unit.team,
        });
    }
    effects
}

/// Applies one round of damage and healing to every unit at once.
/// Each unit takes the summed `strength` of adjacent enemies and gains the summed
/// `heal` of adjacent allies. All results are computed before any are applied, so
/// order does not matter. Units that reach 0 health are removed.
/// Returns an effect for each unit whose health changed or that died.
pub fn apply_damage(board: &mut Board) -> Vec<Effect> {
    let mut updates = Vec::new();
    let mut effects = Vec::new();
    for row in 0..board.height() {
        for col in 0..board.width() {
            let Some(unit) = board.get(row, col) else {
                continue;
            };
            let mut damage = 0;
            let mut heal = 0;
            for (r, c) in board.neighbors(row, col) {
                let Some(other) = board.get(r, c) else {
                    continue;
                };
                if other.team == unit.team {
                    heal += effective_heal(board, (r, c), other);
                } else {
                    damage += effective_strength(board, (r, c), other);
                }
            }
            let health = next_health(unit.health, unit.kind.stats().max_health, damage, heal);
            let updated = (health > 0).then_some(Unit { health, ..unit });
            updates.push((row, col, updated));
            let kind = match health.cmp(&unit.health) {
                _ if health == 0 => Some(EffectKind::Death),
                std::cmp::Ordering::Less => Some(EffectKind::Damage),
                std::cmp::Ordering::Greater => Some(EffectKind::Heal),
                std::cmp::Ordering::Equal => None,
            };
            if let Some(kind) = kind {
                effects.push(Effect {
                    row,
                    col,
                    kind,
                    team: unit.team,
                });
            }
        }
    }
    for (row, col, unit) in updates {
        board.set(row, col, unit);
    }
    effects
}

/// Chebyshev distance between two `(row, col)` cells: diagonal steps count as 1.
pub fn distance(a: (usize, usize), b: (usize, usize)) -> usize {
    let row_gap = a.0.abs_diff(b.0);
    let col_gap = a.1.abs_diff(b.1);
    row_gap.max(col_gap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::UnitKind;

    /// Always takes the first of any tied cells, so tests are deterministic.
    fn first(_: usize) -> usize {
        0
    }

    /// Always takes the last of any tied cells.
    fn last(n: usize) -> usize {
        n - 1
    }

    #[test]
    fn straight_line() {
        assert_eq!(distance((0, 0), (3, 0)), 3);
    }

    #[test]
    fn diagonal() {
        assert_eq!(distance((2, 2), (4, 4)), 2);
    }

    #[test]
    fn max_gap() {
        assert_eq!(distance((2, 2), (4, 5)), 3);
    }

    #[test]
    fn same_cell() {
        assert_eq!(distance((2, 2), (2, 2)), 0);
    }

    fn unit(team: Team) -> Unit {
        Unit {
            team,
            kind: UnitKind::Basic,
            health: UnitKind::Basic.stats().max_health,
        }
    }

    #[test]
    fn advance_prefers_nearest_cell() {
        let board = Board::new(5, 5);
        // From (4, 4) toward (0, 0), (3, 3) is the unique best (distance 3).
        assert_eq!(
            pick_cell(&board, (4, 4), (0, 0), Action::Advance, &mut first),
            Some((3, 3))
        );
    }

    #[test]
    fn advance_tie_prefers_straight_line_then_lowest_row() {
        let mut board = Board::new(5, 5);
        // From (2, 2) toward (2, 4): (1, 3), (2, 3), (3, 3) all reach distance 1,
        // but (2, 3) is on the straight line.
        assert_eq!(
            pick_cell(&board, (2, 2), (2, 4), Action::Advance, &mut first),
            Some((2, 3))
        );
        // With the straight cell taken, the other two tie: lowest row wins.
        board.set(2, 3, Some(unit(Team::Red)));
        assert_eq!(
            pick_cell(&board, (2, 2), (2, 4), Action::Advance, &mut first),
            Some((1, 3))
        );
    }

    #[test]
    fn advance_has_no_vertical_bias() {
        let board = Board::new(5, 5);
        // Mirror images: straight down the column and straight across the row.
        assert_eq!(
            pick_cell(&board, (0, 2), (4, 2), Action::Advance, &mut first),
            Some((1, 2))
        );
        assert_eq!(
            pick_cell(&board, (2, 0), (2, 4), Action::Advance, &mut first),
            Some((2, 1))
        );
    }

    #[test]
    fn nearest_enemy_tie_prefers_straight_line() {
        let mut board = Board::new(5, 5);
        board.set(2, 2, Some(unit(Team::Red)));
        // Both at Chebyshev distance 2; (2, 4) is straight, (0, 3) is not.
        board.set(0, 3, Some(unit(Team::Blue)));
        board.set(2, 4, Some(unit(Team::Blue)));
        assert_eq!(
            nearest_enemy(&board, (2, 2), Team::Red, &mut first),
            Some((2, 4))
        );
    }

    #[test]
    fn advance_none_when_blocked() {
        let mut board = Board::new(3, 3);
        // From (2, 0) toward (0, 2), only (1, 1) is strictly closer.
        assert_eq!(
            pick_cell(&board, (2, 0), (0, 2), Action::Advance, &mut first),
            Some((1, 1))
        );
        board.set(1, 1, Some(unit(Team::Blue)));
        assert_eq!(
            pick_cell(&board, (2, 0), (0, 2), Action::Advance, &mut first),
            None
        );
    }

    #[test]
    fn sidestep_keeps_same_distance() {
        let mut board = Board::new(5, 5);
        // From (2, 0) toward (2, 4), distance 4: only (1, 0) and (3, 0) match.
        assert_eq!(
            pick_cell(&board, (2, 0), (2, 4), Action::Sidestep, &mut first),
            Some((1, 0))
        );
        board.set(1, 0, Some(unit(Team::Red)));
        assert_eq!(
            pick_cell(&board, (2, 0), (2, 4), Action::Sidestep, &mut first),
            Some((3, 0))
        );
    }

    #[test]
    fn sidestep_none_when_no_equal_cell() {
        let mut board = Board::new(3, 3);
        // From (1, 1) toward (1, 2), distance 1: equal cells are the four below.
        for (r, c) in [(0, 1), (0, 2), (2, 1), (2, 2)] {
            board.set(r, c, Some(unit(Team::Red)));
        }
        assert_eq!(
            pick_cell(&board, (1, 1), (1, 2), Action::Sidestep, &mut first),
            None
        );
    }

    #[test]
    fn hold_never_moves() {
        let board = Board::new(5, 5);
        assert_eq!(
            pick_cell(&board, (2, 2), (2, 4), Action::Hold, &mut first),
            None
        );
    }

    fn unit_with_health(team: Team, health: u32) -> Unit {
        Unit {
            health,
            ..unit(team)
        }
    }

    #[test]
    fn next_health_damage_reduces() {
        assert_eq!(next_health(4, 4, 2, 0), 2);
    }

    #[test]
    fn next_health_floors_at_zero() {
        assert_eq!(next_health(1, 4, 5, 0), 0);
    }

    #[test]
    fn next_health_heal_offsets_damage() {
        assert_eq!(next_health(3, 4, 2, 1), 2);
    }

    #[test]
    fn next_health_heal_caps_at_max() {
        assert_eq!(next_health(3, 4, 0, 5), 4);
    }

    #[test]
    fn apply_damage_hits_adjacent_enemy() {
        let mut board = Board::new(3, 3);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 1, Some(unit(Team::Blue)));
        apply_damage(&mut board);
        assert_eq!(board.get(0, 0).unwrap().health, 3);
        assert_eq!(board.get(0, 1).unwrap().health, 3);
    }

    #[test]
    fn apply_damage_ignores_distant_and_allied_units() {
        let mut board = Board::new(5, 5);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 1, Some(unit(Team::Red)));
        board.set(4, 4, Some(unit(Team::Blue)));
        apply_damage(&mut board);
        assert_eq!(board.get(0, 0), Some(unit(Team::Red)));
        assert_eq!(board.get(0, 1), Some(unit(Team::Red)));
        assert_eq!(board.get(4, 4), Some(unit(Team::Blue)));
    }

    #[test]
    fn apply_damage_sums_adjacent_enemies() {
        let mut board = Board::new(3, 3);
        board.set(1, 1, Some(unit(Team::Red)));
        board.set(0, 0, Some(unit(Team::Blue)));
        board.set(2, 2, Some(unit(Team::Blue)));
        apply_damage(&mut board);
        assert_eq!(board.get(1, 1).unwrap().health, 2);
    }

    #[test]
    fn apply_damage_is_simultaneous() {
        let mut board = Board::new(3, 3);
        board.set(0, 0, Some(unit_with_health(Team::Red, 1)));
        board.set(0, 1, Some(unit_with_health(Team::Blue, 1)));
        apply_damage(&mut board);
        // Both die; neither is spared by the other's earlier removal.
        assert_eq!(board.get(0, 0), None);
        assert_eq!(board.get(0, 1), None);
    }

    #[test]
    fn apply_damage_removes_dead_only() {
        let mut board = Board::new(3, 3);
        board.set(0, 0, Some(unit_with_health(Team::Red, 1)));
        board.set(0, 1, Some(unit(Team::Blue)));
        apply_damage(&mut board);
        assert_eq!(board.get(0, 0), None);
        assert_eq!(board.get(0, 1).unwrap().health, 3);
    }

    fn kind_unit(team: Team, kind: UnitKind) -> Unit {
        Unit {
            team,
            kind,
            health: kind.stats().max_health,
        }
    }

    #[test]
    fn commander_boosts_touching_allies() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 1, Some(unit(Team::Blue)));
        board.set(0, 2, Some(kind_unit(Team::Blue, UnitKind::Commander)));
        apply_damage(&mut board);
        // The Blue Basic hits for 1 + 1 (commander next to it).
        assert_eq!(board.get(0, 0).unwrap().health, 2);
        // The Red Basic has no commander, so it hits for 1.
        assert_eq!(board.get(0, 1).unwrap().health, 3);
    }

    #[test]
    fn is_boosted_for_fighters_next_to_a_commander() {
        let mut board = Board::new(4, 1);
        board.set(0, 0, Some(unit(Team::Blue)));
        board.set(0, 1, Some(kind_unit(Team::Blue, UnitKind::Commander)));
        board.set(0, 3, Some(unit(Team::Blue)));
        assert!(is_boosted(&board, 0, 0));
        // The commander does not boost itself, and a distant unit is not boosted.
        assert!(!is_boosted(&board, 0, 1));
        assert!(!is_boosted(&board, 0, 3));
    }

    #[test]
    fn is_boosted_false_for_harmless_enemy_and_empty_cells() {
        let mut board = Board::new(4, 1);
        board.set(0, 0, Some(kind_unit(Team::Blue, UnitKind::Priest)));
        board.set(0, 1, Some(kind_unit(Team::Blue, UnitKind::Commander)));
        board.set(0, 2, Some(unit(Team::Red)));
        assert!(!is_boosted(&board, 0, 0)); // no damage or healing to boost
        assert!(!is_boosted(&board, 0, 2)); // enemy of the commander
        assert!(!is_boosted(&board, 0, 3)); // empty
    }

    #[test]
    fn commander_boosts_a_healers_healing() {
        let heal_received = |with_commander: bool| {
            let mut board = Board::new(4, 1);
            board.set(0, 0, Some(unit_with_health(Team::Blue, 1)));
            board.set(0, 1, Some(kind_unit(Team::Blue, UnitKind::Healer)));
            if with_commander {
                board.set(0, 2, Some(kind_unit(Team::Blue, UnitKind::Commander)));
            }
            apply_damage(&mut board);
            board.get(0, 0).unwrap().health
        };
        // Starting at 1 health: +1 from a Healer, +2 once a Commander touches it.
        assert_eq!(heal_received(false), 2);
        assert_eq!(heal_received(true), 3);
    }

    #[test]
    fn healer_next_to_a_commander_counts_as_boosted() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(kind_unit(Team::Blue, UnitKind::Healer)));
        assert!(!is_boosted(&board, 0, 0));
        board.set(0, 1, Some(kind_unit(Team::Blue, UnitKind::Commander)));
        assert!(is_boosted(&board, 0, 0));
    }

    #[test]
    fn commander_does_not_boost_itself() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(kind_unit(Team::Blue, UnitKind::Commander)));
        board.set(0, 1, Some(unit(Team::Red)));
        apply_damage(&mut board);
        assert_eq!(board.get(0, 1).unwrap().health, 3);
    }

    #[test]
    fn commander_does_not_arm_harmless_allies() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 1, Some(kind_unit(Team::Blue, UnitKind::Healer)));
        board.set(0, 2, Some(kind_unit(Team::Blue, UnitKind::Commander)));
        apply_damage(&mut board);
        assert_eq!(board.get(0, 0).unwrap().health, 4);
    }

    #[test]
    fn priest_converts_adjacent_enemy_and_keeps_health() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(kind_unit(Team::Red, UnitKind::Priest)));
        board.set(0, 1, Some(unit_with_health(Team::Blue, 2)));
        apply_conversion(&mut board);
        let flipped = board.get(0, 1).unwrap();
        assert_eq!(flipped.team, Team::Red);
        assert_eq!(flipped.kind, UnitKind::Basic);
        assert_eq!(flipped.health, 2);
    }

    #[test]
    fn priest_converts_every_touching_enemy() {
        let mut board = Board::new(3, 3);
        board.set(1, 1, Some(kind_unit(Team::Red, UnitKind::Priest)));
        board.set(0, 1, Some(unit(Team::Blue)));
        board.set(1, 0, Some(unit_with_health(Team::Blue, 2)));
        board.set(2, 2, Some(kind_unit(Team::Blue, UnitKind::Healer)));
        let effects = apply_conversion(&mut board);
        for (row, col) in [(0, 1), (1, 0), (2, 2)] {
            assert_eq!(board.get(row, col).unwrap().team, Team::Red);
        }
        // Each keeps its own kind and health.
        assert_eq!(board.get(1, 0).unwrap().health, 2);
        assert_eq!(board.get(2, 2).unwrap().kind, UnitKind::Healer);
        assert_eq!(effects.len(), 3);
    }

    #[test]
    fn cell_touched_by_two_priests_of_one_team_is_reported_once() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(kind_unit(Team::Red, UnitKind::Priest)));
        board.set(0, 1, Some(unit(Team::Blue)));
        board.set(0, 2, Some(kind_unit(Team::Red, UnitKind::Priest)));
        let effects = apply_conversion(&mut board);
        assert_eq!(board.get(0, 1).unwrap().team, Team::Red);
        assert_eq!(effects.len(), 1);
    }

    #[test]
    fn priest_only_converts_enemies_not_its_own_team() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(kind_unit(Team::Red, UnitKind::Priest)));
        board.set(0, 1, Some(unit(Team::Red)));
        board.set(0, 2, Some(kind_unit(Team::Blue, UnitKind::Priest)));
        // The Red basic touches both priests: Red's is an ally and is ignored,
        // so only the Blue priest acts on it.
        apply_conversion(&mut board);
        assert_eq!(board.get(0, 1).unwrap().team, Team::Blue);
    }

    #[test]
    fn priest_ignores_allies_and_distant_enemies() {
        let mut board = Board::new(5, 1);
        board.set(0, 0, Some(kind_unit(Team::Red, UnitKind::Priest)));
        board.set(0, 1, Some(unit(Team::Red)));
        board.set(0, 4, Some(unit(Team::Blue)));
        apply_conversion(&mut board);
        assert_eq!(board.get(0, 1).unwrap().team, Team::Red);
        assert_eq!(board.get(0, 4).unwrap().team, Team::Blue);
    }

    #[test]
    fn non_priests_never_convert() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 1, Some(unit(Team::Blue)));
        apply_conversion(&mut board);
        assert_eq!(board.get(0, 1).unwrap().team, Team::Blue);
    }

    #[test]
    fn opposing_priests_swap_sides() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(kind_unit(Team::Red, UnitKind::Priest)));
        board.set(0, 1, Some(kind_unit(Team::Blue, UnitKind::Priest)));
        apply_conversion(&mut board);
        assert_eq!(board.get(0, 0).unwrap().team, Team::Blue);
        assert_eq!(board.get(0, 1).unwrap().team, Team::Red);
    }

    fn kinds_at(effects: &[Effect], row: usize, col: usize) -> Vec<EffectKind> {
        effects
            .iter()
            .filter(|e| e.row == row && e.col == col)
            .map(|e| e.kind)
            .collect()
    }

    #[test]
    fn damage_effect_for_each_unit_hit() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 1, Some(unit(Team::Blue)));
        let effects = apply_damage(&mut board);
        assert_eq!(kinds_at(&effects, 0, 0), vec![EffectKind::Damage]);
        assert_eq!(kinds_at(&effects, 0, 1), vec![EffectKind::Damage]);
    }

    #[test]
    fn death_effect_replaces_damage_when_unit_dies() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(unit_with_health(Team::Red, 1)));
        board.set(0, 1, Some(unit(Team::Blue)));
        let effects = apply_damage(&mut board);
        assert_eq!(kinds_at(&effects, 0, 0), vec![EffectKind::Death]);
        assert_eq!(kinds_at(&effects, 0, 1), vec![EffectKind::Damage]);
    }

    #[test]
    fn death_effect_carries_the_dead_units_team() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(unit_with_health(Team::Red, 1)));
        board.set(0, 1, Some(unit_with_health(Team::Blue, 1)));
        let effects = apply_damage(&mut board);
        let team_of = |col| effects.iter().find(|e| e.col == col).unwrap().team;
        assert_eq!(team_of(0), Team::Red);
        assert_eq!(team_of(1), Team::Blue);
    }

    #[test]
    fn heal_effect_when_health_rises() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(kind_unit(Team::Blue, UnitKind::Healer)));
        board.set(0, 1, Some(unit_with_health(Team::Blue, 2)));
        let effects = apply_damage(&mut board);
        assert_eq!(kinds_at(&effects, 0, 1), vec![EffectKind::Heal]);
        assert!(kinds_at(&effects, 0, 0).is_empty());
    }

    #[test]
    fn no_effect_when_health_is_unchanged() {
        let mut board = Board::new(3, 1);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 2, Some(unit(Team::Blue)));
        assert!(apply_damage(&mut board).is_empty());
    }

    #[test]
    fn convert_effect_on_flipped_cell() {
        let mut board = Board::new(2, 1);
        board.set(0, 0, Some(kind_unit(Team::Red, UnitKind::Priest)));
        board.set(0, 1, Some(unit(Team::Blue)));
        let effects = apply_conversion(&mut board);
        assert_eq!(kinds_at(&effects, 0, 1), vec![EffectKind::Convert]);
        assert_eq!(effects.len(), 1);
    }

    #[test]
    fn take_turn_advances_toward_enemy() {
        let mut board = Board::new(5, 5);
        board.set(2, 0, Some(unit(Team::Red)));
        board.set(2, 4, Some(unit(Team::Blue)));
        assert_eq!(take_turn(&mut board, (2, 0), &mut first), Some((2, 1)));
        assert_eq!(board.get(2, 0), None);
        assert_eq!(board.get(2, 1), Some(unit(Team::Red)));
    }

    #[test]
    fn take_turn_falls_through_to_sidestep() {
        let mut board = Board::new(3, 3);
        board.set(2, 0, Some(unit(Team::Red)));
        board.set(0, 2, Some(unit(Team::Blue)));
        // Ally blocks the only closer cell, so Advance fails.
        board.set(1, 1, Some(unit(Team::Red)));
        assert_eq!(take_turn(&mut board, (2, 0), &mut first), Some((1, 0)));
        assert_eq!(board.get(1, 0), Some(unit(Team::Red)));
    }

    #[test]
    fn take_turn_none_without_unit_or_enemy() {
        let mut board = Board::new(3, 3);
        assert_eq!(take_turn(&mut board, (1, 1), &mut first), None);
        board.set(1, 1, Some(unit(Team::Red)));
        assert_eq!(take_turn(&mut board, (1, 1), &mut first), None);
        assert_eq!(board.get(1, 1), Some(unit(Team::Red)));
    }

    #[test]
    fn take_turn_reaches_across_the_board() {
        // Basic bands cover any distance, so even the far side is chased.
        let mut board = Board::new(10, 10);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 9, Some(unit(Team::Blue)));
        assert_eq!(take_turn(&mut board, (0, 0), &mut first), Some((0, 1)));
    }

    #[test]
    fn walk_bands_hold_stops_the_walk() {
        let board = Board::new(5, 5);
        let bands = [
            Band {
                max_dist: 4,
                action: Action::Hold,
            },
            Band {
                max_dist: 4,
                action: Action::Advance,
            },
        ];
        assert_eq!(walk_bands(&board, (2, 0), (2, 4), &bands, &mut first), None);
    }

    #[test]
    fn walk_bands_skips_bands_out_of_range() {
        let board = Board::new(5, 5);
        let bands = [
            Band {
                max_dist: 1,
                action: Action::Hold,
            },
            Band {
                max_dist: 4,
                action: Action::Advance,
            },
        ];
        assert_eq!(
            walk_bands(&board, (2, 0), (2, 4), &bands, &mut first),
            Some((2, 1))
        );
    }

    #[test]
    fn sidestep_tie_is_settled_by_pick() {
        let board = Board::new(5, 5);
        // From (2, 0) toward (2, 4): (1, 0) and (3, 0) are exactly tied.
        assert_eq!(
            pick_cell(&board, (2, 0), (2, 4), Action::Sidestep, &mut first),
            Some((1, 0))
        );
        assert_eq!(
            pick_cell(&board, (2, 0), (2, 4), Action::Sidestep, &mut last),
            Some((3, 0))
        );
    }

    #[test]
    fn advance_tie_is_settled_by_pick() {
        let mut board = Board::new(5, 5);
        board.set(2, 3, Some(unit(Team::Red)));
        // With the straight cell taken, (1, 3) and (3, 3) are exactly tied.
        assert_eq!(
            pick_cell(&board, (2, 2), (2, 4), Action::Advance, &mut first),
            Some((1, 3))
        );
        assert_eq!(
            pick_cell(&board, (2, 2), (2, 4), Action::Advance, &mut last),
            Some((3, 3))
        );
    }

    #[test]
    fn nearest_enemy_tie_is_settled_by_pick() {
        let mut board = Board::new(5, 5);
        board.set(2, 2, Some(unit(Team::Red)));
        // Same distance and same straightness: (0, 2) and (4, 2).
        board.set(0, 2, Some(unit(Team::Blue)));
        board.set(4, 2, Some(unit(Team::Blue)));
        assert_eq!(
            nearest_enemy(&board, (2, 2), Team::Red, &mut first),
            Some((0, 2))
        );
        assert_eq!(
            nearest_enemy(&board, (2, 2), Team::Red, &mut last),
            Some((4, 2))
        );
    }

    #[test]
    fn pick_is_only_consulted_for_real_ties() {
        let board = Board::new(5, 5);
        let mut calls = 0;
        let mut counting = |_: usize| {
            calls += 1;
            0
        };
        // A unique best cell, so there is nothing to choose between.
        pick_cell(&board, (4, 4), (0, 0), Action::Advance, &mut counting);
        assert_eq!(calls, 0);
    }

    #[test]
    fn nearest_enemy_none_when_no_enemies() {
        let mut board = Board::new(5, 5);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(4, 4, Some(unit(Team::Red)));
        assert_eq!(nearest_enemy(&board, (0, 0), Team::Red, &mut first), None);
    }

    #[test]
    fn nearest_enemy_picks_closest() {
        let mut board = Board::new(10, 10);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 8, Some(unit(Team::Blue)));
        board.set(5, 5, Some(unit(Team::Blue)));
        assert_eq!(
            nearest_enemy(&board, (0, 0), Team::Red, &mut first),
            Some((5, 5))
        );
    }

    #[test]
    fn nearest_enemy_ignores_allies() {
        let mut board = Board::new(5, 5);
        board.set(0, 0, Some(unit(Team::Red)));
        board.set(0, 1, Some(unit(Team::Red)));
        board.set(4, 4, Some(unit(Team::Blue)));
        assert_eq!(
            nearest_enemy(&board, (0, 0), Team::Red, &mut first),
            Some((4, 4))
        );
    }

    #[test]
    fn nearest_enemy_tie_lowest_row_then_column() {
        let mut board = Board::new(5, 5);
        board.set(2, 2, Some(unit(Team::Red)));
        // All at distance 2 from (2, 2).
        board.set(4, 0, Some(unit(Team::Blue)));
        board.set(0, 4, Some(unit(Team::Blue)));
        board.set(0, 2, Some(unit(Team::Blue)));
        assert_eq!(
            nearest_enemy(&board, (2, 2), Team::Red, &mut first),
            Some((0, 2))
        );
        board.set(0, 2, None);
        assert_eq!(
            nearest_enemy(&board, (2, 2), Team::Red, &mut first),
            Some((0, 4))
        );
    }
}
