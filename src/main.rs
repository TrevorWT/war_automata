use macroquad::prelude::*;
use war_automata::{
    Battle, Board, Effect, EffectKind, Outcome, ROUND_LIMIT, Setup, Team, UnitKind, is_boosted,
};

const SQUARES: usize = 32;
const PAD: f32 = 12.;
const THICKNESS: f32 = 1.;
const ROUND_SECONDS: f32 = 0.15;
const FLASH_SECONDS: f32 = 0.25;
const STATUS_FONT: u16 = 22;
const STATUS_LINE: f32 = 28.;
const STATUS_LINES: f32 = 5.;
const REPEAT_MAX: u32 = 1000;
/// Seconds per frame spent simulating while repeating, so the window stays responsive.
const FRAME_BUDGET: f64 = 0.01;
const BUDGET: u32 = 30;
const BUDGET_STEP: u32 = 5;
const BUDGET_MAX: u32 = 200;
const NUMBER_KEYS: [KeyCode; 4] = [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4];

const BACKGROUND: Color = Color::new(0.11, 0.12, 0.14, 1.);
const BOARD_COLOR: Color = Color::new(0.97, 0.97, 0.95, 1.);
const GRID_COLOR: Color = Color::new(0.78, 0.78, 0.76, 1.);
const TEXT_COLOR: Color = Color::new(0.9, 0.9, 0.88, 1.);
const RED_UNIT: Color = Color::new(0.82, 0.2, 0.2, 1.);
const BLUE_UNIT: Color = Color::new(0.2, 0.4, 0.82, 1.);

fn window_conf() -> Conf {
    Conf {
        window_title: "War Automata".to_owned(),
        window_width: 900,
        window_height: 850,
        // Render at the monitor's real resolution instead of stretching a
        // low-resolution image, which is what made the grid look blurry.
        high_dpi: true,
        ..Default::default()
    }
}

/// All sizes are whole numbers so lines and squares land on exact pixels.
struct Layout {
    offset_x: f32,
    offset_y: f32,
    board_size: f32,
    sq_size: f32,
    status_y: f32,
}

fn compute_layout() -> Layout {
    let status_height = STATUS_LINE * STATUS_LINES;
    let room_x = screen_width() - 2. * PAD;
    let room_y = screen_height() - 3. * PAD - status_height;
    // Whole pixels per square, so the grid never straddles pixel boundaries.
    let sq_size = (room_x.min(room_y) / SQUARES as f32).floor().max(1.);
    let board_size = sq_size * SQUARES as f32;
    let offset_x = ((screen_width() - board_size) / 2.).floor();
    let offset_y = PAD;
    Layout {
        offset_x,
        offset_y,
        board_size,
        sq_size,
        status_y: offset_y + board_size + PAD,
    }
}

/// Grid cell `(row, col)` under screen position `(x, y)`, if it is on the board.
fn cell_at(layout: &Layout, x: f32, y: f32) -> Option<(usize, usize)> {
    let col = (x - layout.offset_x) / layout.sq_size;
    let row = (y - layout.offset_y) / layout.sq_size;
    let limit = SQUARES as f32;
    if col < 0. || row < 0. || col >= limit || row >= limit {
        return None;
    }
    Some((row as usize, col as usize))
}

fn draw_grid(layout: &Layout) {
    draw_rectangle(
        layout.offset_x,
        layout.offset_y,
        layout.board_size,
        layout.board_size,
        BOARD_COLOR,
    );

    // Lines are thin rectangles on whole pixels; the last pair sits inside the edge.
    for i in 0..=SQUARES {
        let line = layout.sq_size * i as f32;
        let inset = if i == SQUARES { THICKNESS } else { 0. };
        draw_rectangle(
            layout.offset_x,
            layout.offset_y + line - inset,
            layout.board_size,
            THICKNESS,
            GRID_COLOR,
        );
        draw_rectangle(
            layout.offset_x + line - inset,
            layout.offset_y,
            THICKNESS,
            layout.board_size,
            GRID_COLOR,
        );
    }
}

/// Tints each half in its team's color so it is clear where clicks place units.
fn draw_sides(layout: &Layout) {
    let half = layout.sq_size * (SQUARES / 2) as f32;
    draw_rectangle(
        layout.offset_x,
        layout.offset_y,
        half,
        layout.board_size,
        Color::new(0.2, 0.4, 0.82, 0.1),
    );
    draw_rectangle(
        layout.offset_x + half,
        layout.offset_y,
        half,
        layout.board_size,
        Color::new(0.82, 0.2, 0.2, 0.1),
    );
}

fn team_color(team: Team) -> Color {
    match team {
        Team::Red => RED_UNIT,
        Team::Blue => BLUE_UNIT,
    }
}

/// Outline color that marks a unit kind (`None` for Basic).
fn kind_outline(kind: UnitKind) -> Option<Color> {
    match kind {
        UnitKind::Basic => None,
        UnitKind::Healer => Some(GREEN),
        UnitKind::Priest => Some(GOLD),
        UnitKind::Commander => Some(BLACK),
    }
}

/// One-line summary of what a kind does, shown while placing.
fn kind_note(kind: UnitKind) -> &'static str {
    match kind {
        UnitKind::Basic => "fights touching enemies",
        UnitKind::Healer => "heals touching allies (white outline)",
        UnitKind::Priest => "converts all touching enemies (gold outline)",
        UnitKind::Commander => "tough, +1 damage and +1 healing to touching allies (black outline)",
    }
}

fn draw_units(board: &Board, layout: &Layout) {
    // Insets scale with the square so units look right at any window size.
    let inset = (layout.sq_size / 8.).floor().max(1.);
    let outline = (layout.sq_size / 5.).floor().max(3.);
    let font = ((layout.sq_size * 0.6).floor() as u16).max(8);
    for row in 0..board.height() {
        for col in 0..board.width() {
            let Some(unit) = board.get(row, col) else {
                continue;
            };
            let x = layout.offset_x + col as f32 * layout.sq_size;
            let y = layout.offset_y + row as f32 * layout.sq_size;
            draw_rectangle(
                x + inset,
                y + inset,
                layout.sq_size - 2. * inset,
                layout.sq_size - 2. * inset,
                team_color(unit.team),
            );
            // Non-basic kinds get a colored outline so they stand out from Basic.
            if let Some(color) = kind_outline(unit.kind) {
                draw_rectangle_lines(
                    x + inset,
                    y + inset,
                    layout.sq_size - 2. * inset,
                    layout.sq_size - 2. * inset,
                    outline,
                    color,
                );
            }
            // Current health, centered on the unit.
            let label = unit.health.to_string();
            let size = measure_text(&label, None, font, 1.);
            draw_text(
                &label,
                (x + (layout.sq_size - size.width) / 2.).floor(),
                (y + (layout.sq_size + size.offset_y) / 2.).floor(),
                font as f32,
                // Gold health marks a unit a Commander is boosting.
                if is_boosted(board, row, col) {
                    GOLD
                } else {
                    WHITE
                },
            );
        }
    }
}

fn battle_status(battle: &Battle, running: bool, done: u32, total: u32) -> Vec<String> {
    let state = match battle.outcome() {
        Outcome::Won(Team::Red) => "Red wins",
        Outcome::Won(Team::Blue) => "Blue wins",
        Outcome::Draw => "Draw",
        Outcome::Ongoing if running => "Running",
        Outcome::Ongoing => "Paused",
    };
    // Only repeated runs show how far along they are.
    let game = match total {
        1 => String::new(),
        _ if done >= total => format!("Finished {total} games | "),
        _ => format!("Game {}/{} | ", done + 1, total),
    };
    vec![
        format!(
            "{game}Round {}/{} | {}",
            battle.round_count(),
            ROUND_LIMIT,
            state
        ),
        "Space: run/pause  N: step  R: restart  S: setup".to_string(),
    ]
}

fn placing_status(setup: &Setup, kind: UnitKind, repeat: u32) -> Vec<String> {
    vec![
        format!(
            "Setup | Blue {}/{} | Red {}/{}",
            setup.budget(Team::Blue),
            setup.limit(),
            setup.budget(Team::Red),
            setup.limit()
        ),
        format!(
            "Unit: {} (cost {}): {}",
            kind.name(),
            kind.stats().cost,
            kind_note(kind)
        ),
        format!(
            "Tab/1-{}: unit  Left: add  Right: remove  +/-: budget",
            UnitKind::ALL.len()
        ),
        format!("[/]: repeat x{repeat} (Shift: 10)  Space: start"),
    ]
}

/// Draws the status text in the strip below the board.
fn draw_status(lines: &[String], layout: &Layout) {
    for (i, line) in lines.iter().enumerate() {
        draw_text(
            line,
            layout.offset_x,
            layout.status_y + STATUS_LINE * (i as f32 + 1.) - 6.,
            STATUS_FONT as f32,
            TEXT_COLOR,
        );
    }
}

/// Switches the selected unit kind with Tab (cycle) or the number keys.
fn pick_kind(mut index: usize) -> usize {
    if is_key_pressed(KeyCode::Tab) {
        index = (index + 1) % UnitKind::ALL.len();
    }
    for (i, key) in NUMBER_KEYS.iter().enumerate().take(UnitKind::ALL.len()) {
        if is_key_pressed(*key) {
            index = i;
        }
    }
    index
}

/// Results of finished battles, kept across resets so averages can build up.
#[derive(Debug, Default, PartialEq, Eq)]
struct Tally {
    red: u32,
    blue: u32,
    draws: u32,
}

impl Tally {
    /// Adds a finished battle. An ongoing battle is not counted.
    fn record(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Won(Team::Red) => self.red += 1,
            Outcome::Won(Team::Blue) => self.blue += 1,
            Outcome::Draw => self.draws += 1,
            Outcome::Ongoing => {}
        }
    }

    fn games(&self) -> u32 {
        self.red + self.blue + self.draws
    }

    /// Whole-number percent of all games, or 0 before any have finished.
    fn percent(&self, count: u32) -> u32 {
        match self.games() {
            0 => 0,
            games => (count as f32 * 100. / games as f32).round() as u32,
        }
    }

    fn summary(&self) -> String {
        format!(
            "Wins - Red {} ({}%) | Blue {} ({}%) | Draws {} | Games {} | C: clear",
            self.red,
            self.percent(self.red),
            self.blue,
            self.percent(self.blue),
            self.draws,
            self.games()
        )
    }
}

/// A short-lived highlight on a cell, so rounds are easy to follow.
struct Flash {
    effect: Effect,
    /// Seconds since the flash started.
    age: f32,
}

/// A battle on a copy of the placed army. Each call advances `seed`, so every
/// game breaks ties differently even when many start within the same instant.
/// The first mover alternates by game number: game 1 starts with Red, game 2
/// with Blue, and so on, so neither side keeps the opening advantage.
fn new_battle(setup: &Setup, seed: &mut u64, games_played: u32) -> Battle {
    *seed = seed.wrapping_add(1);
    let first = if games_played % 2 == 0 {
        Team::Red
    } else {
        Team::Blue
    };
    Battle::with_seed(setup.board().clone(), *seed).with_first_mover(first)
}

/// Plays one round and starts a flash for everything that happened in it.
fn play_round(battle: &mut Battle, flashes: &mut Vec<Flash>) {
    let before = battle.round_count();
    battle.round();
    // A finished battle ignores `round`, so don't replay the last round's effects.
    if battle.round_count() != before {
        flashes.extend(
            battle
                .effects()
                .iter()
                .map(|&effect| Flash { effect, age: 0. }),
        );
    }
}

fn flash_color(kind: EffectKind) -> Color {
    match kind {
        EffectKind::Damage => WHITE,
        EffectKind::Heal => Color::new(0.2, 0.85, 0.35, 1.),
        EffectKind::Death => WHITE, // overridden by the team color in `draw_flashes`
        EffectKind::Convert => GOLD,
    }
}

/// Draws each flash as a square over its cell that fades out. Deaths grow
/// past the cell edges, since the unit is already gone.
fn draw_flashes(flashes: &[Flash], layout: &Layout) {
    for flash in flashes {
        let progress = flash.age / FLASH_SECONDS;
        // A death burst takes the color of the team that lost the unit.
        let mut color = match flash.effect.kind {
            EffectKind::Death => team_color(flash.effect.team),
            kind => flash_color(kind),
        };
        color.a = (1. - progress) * 0.85;
        let grow = if flash.effect.kind == EffectKind::Death {
            progress * layout.sq_size * 0.5
        } else {
            0.
        };
        draw_rectangle(
            layout.offset_x + flash.effect.col as f32 * layout.sq_size - grow,
            layout.offset_y + flash.effect.row as f32 * layout.sq_size - grow,
            layout.sq_size + 2. * grow,
            layout.sq_size + 2. * grow,
            color,
        );
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut setup = Setup::new(SQUARES, SQUARES, BUDGET);
    let mut kind_index = 0;
    // `None` while placing units; `Some` once the battle has started.
    let mut battle: Option<Battle> = None;
    let mut running = false;
    let mut timer = 0.;
    let mut flashes: Vec<Flash> = Vec::new();
    // Win record across battles. It survives resets; press C to clear it.
    let mut tally = Tally::default();
    // Whether the current battle's result is already in the tally.
    let mut recorded = false;
    // How many games Space or R plays in a row, and how far the current run is.
    let mut repeat_count: u32 = 1;
    let mut batch_total: u32 = 1;
    let mut batch_done: u32 = 0;
    // Each game takes the next seed, so quick back-to-back games still differ.
    let mut seed = (macroquad::miniquad::date::now() * 1000.) as u64;
    loop {
        let layout = compute_layout();
        let dt = get_frame_time();
        for flash in &mut flashes {
            flash.age += dt;
        }
        flashes.retain(|flash| flash.age < FLASH_SECONDS);

        if is_key_pressed(KeyCode::C) {
            tally = Tally::default();
        }

        let mut status = match battle.as_mut() {
            None => {
                kind_index = pick_kind(kind_index);
                let kind = UnitKind::ALL[kind_index];
                let (mx, my) = mouse_position();
                if let Some((row, col)) = cell_at(&layout, mx, my) {
                    if is_mouse_button_down(MouseButton::Left) {
                        // Wrong side, occupied or over budget: nothing happens.
                        setup.place(row, col, kind).ok();
                    }
                    if is_mouse_button_down(MouseButton::Right) {
                        setup.remove(row, col).ok();
                    }
                }
                if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
                    setup.set_limit((setup.limit() + BUDGET_STEP).min(BUDGET_MAX));
                }
                if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
                    setup.set_limit(setup.limit().saturating_sub(BUDGET_STEP));
                }
                let step = if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    10
                } else {
                    1
                };
                if is_key_pressed(KeyCode::RightBracket) {
                    repeat_count = (repeat_count + step).min(REPEAT_MAX);
                }
                if is_key_pressed(KeyCode::LeftBracket) {
                    repeat_count = repeat_count.saturating_sub(step).max(1);
                }
                if is_key_pressed(KeyCode::Space) {
                    battle = Some(new_battle(&setup, &mut seed, tally.games()));
                    // Several games run straight away; a single game starts paused.
                    running = repeat_count > 1;
                    recorded = false;
                    batch_done = 0;
                    batch_total = repeat_count;
                    timer = 0.;
                    flashes.clear();
                }
                placing_status(&setup, kind, repeat_count)
            }
            Some(b) => {
                if is_key_pressed(KeyCode::Space) {
                    running = !running;
                }
                if is_key_pressed(KeyCode::N) {
                    // Stepping by hand pauses auto-play.
                    running = false;
                    play_round(b, &mut flashes);
                }
                if batch_total > 1 {
                    // Repeating: skip the animation and play as many rounds as
                    // fit in a frame, starting the next game as each one ends.
                    flashes.clear();
                    let start = get_time();
                    while running && get_time() - start < FRAME_BUDGET {
                        if b.outcome() == Outcome::Ongoing {
                            b.round();
                            continue;
                        }
                        if !recorded {
                            tally.record(b.outcome());
                            recorded = true;
                            batch_done += 1;
                        }
                        if batch_done < batch_total {
                            *b = new_battle(&setup, &mut seed, tally.games());
                            recorded = false;
                        } else {
                            running = false;
                        }
                    }
                } else if running {
                    timer += dt;
                    while timer >= ROUND_SECONDS {
                        timer -= ROUND_SECONDS;
                        play_round(b, &mut flashes);
                    }
                }
                // A repeating batch ends its own games above; stopping here would
                // pause it whenever a game finishes as the frame budget runs out.
                if batch_total == 1 && b.outcome() != Outcome::Ongoing {
                    running = false;
                    // Count each finished battle once.
                    if !recorded {
                        tally.record(b.outcome());
                        recorded = true;
                        batch_done += 1;
                    }
                }
                let lines = battle_status(b, running, batch_done, batch_total);
                if is_key_pressed(KeyCode::S) {
                    battle = None;
                    flashes.clear();
                } else if is_key_pressed(KeyCode::R) {
                    // Same army, fresh seed, already running; repeats start over.
                    battle = Some(new_battle(&setup, &mut seed, tally.games()));
                    running = true;
                    recorded = false;
                    batch_done = 0;
                    batch_total = repeat_count;
                    timer = 0.;
                    flashes.clear();
                }
                lines
            }
        };
        status.push(tally.summary());

        clear_background(BACKGROUND);
        draw_grid(&layout);
        match &battle {
            Some(b) => {
                draw_units(b.board(), &layout);
                draw_flashes(&flashes, &layout);
            }
            None => {
                draw_sides(&layout);
                draw_units(setup.board(), &layout);
            }
        }
        draw_status(&status, &layout);
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout {
            offset_x: 100.,
            offset_y: 50.,
            board_size: 320.,
            sq_size: 10.,
            status_y: 382.,
        }
    }

    #[test]
    fn cell_at_maps_inside_the_board() {
        assert_eq!(cell_at(&layout(), 100., 50.), Some((0, 0)));
        assert_eq!(cell_at(&layout(), 125., 63.), Some((1, 2)));
        assert_eq!(cell_at(&layout(), 419., 369.), Some((31, 31)));
    }

    #[test]
    fn tally_counts_each_result_and_ignores_ongoing() {
        let mut tally = Tally::default();
        tally.record(Outcome::Won(Team::Red));
        tally.record(Outcome::Won(Team::Red));
        tally.record(Outcome::Won(Team::Blue));
        tally.record(Outcome::Draw);
        tally.record(Outcome::Ongoing);
        assert_eq!(
            tally,
            Tally {
                red: 2,
                blue: 1,
                draws: 1
            }
        );
        assert_eq!(tally.games(), 4);
    }

    #[test]
    fn tally_percentages() {
        let mut tally = Tally::default();
        // No games yet: no division by zero.
        assert_eq!(tally.percent(0), 0);
        tally.record(Outcome::Won(Team::Red));
        tally.record(Outcome::Won(Team::Red));
        tally.record(Outcome::Won(Team::Blue));
        assert_eq!(tally.percent(tally.red), 67);
        assert_eq!(tally.percent(tally.blue), 33);
    }

    #[test]
    fn cell_at_rejects_outside_the_board() {
        assert_eq!(cell_at(&layout(), 99., 60.), None);
        assert_eq!(cell_at(&layout(), 120., 49.), None);
        assert_eq!(cell_at(&layout(), 420., 60.), None);
        assert_eq!(cell_at(&layout(), 120., 370.), None);
    }
}
