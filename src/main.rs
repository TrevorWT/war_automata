use macroquad::prelude::*;
use war_automata::{Battle, Board, Outcome, ROUND_LIMIT, Setup, Team, UnitKind};

const SQUARES: usize = 32;
const PAD: f32 = 12.;
const THICKNESS: f32 = 1.;
const ROUND_SECONDS: f32 = 0.15;
const STATUS_FONT: u16 = 22;
const STATUS_LINE: f32 = 28.;
const STATUS_LINES: f32 = 3.;
const BUDGET: u32 = 30;
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
        UnitKind::Priest => "converts weakest touching enemy (gold outline)",
        UnitKind::Commander => "tough, +1 damage to touching allies (black outline)",
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
                WHITE,
            );
        }
    }
}

fn battle_status(battle: &Battle, running: bool) -> Vec<String> {
    let state = match battle.outcome() {
        Outcome::Won(Team::Red) => "Red wins",
        Outcome::Won(Team::Blue) => "Blue wins",
        Outcome::Draw => "Draw",
        Outcome::Ongoing if running => "Running",
        Outcome::Ongoing => "Paused",
    };
    vec![
        format!("Round {}/{} | {}", battle.round_count(), ROUND_LIMIT, state),
        "Space: run/pause  N: step  R: back to placing".to_string(),
    ]
}

fn placing_status(setup: &Setup, kind: UnitKind) -> Vec<String> {
    vec![
        format!(
            "Placing | Blue {}/{} | Red {}/{}",
            setup.budget(Team::Blue),
            BUDGET,
            setup.budget(Team::Red),
            BUDGET
        ),
        format!(
            "Unit: {} (cost {}): {}",
            kind.name(),
            kind.stats().cost,
            kind_note(kind)
        ),
        format!(
            "Tab/1-{}: switch  Left: add  Right: remove  Space: start",
            UnitKind::ALL.len()
        ),
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

#[macroquad::main(window_conf)]
async fn main() {
    let mut setup = Setup::new(SQUARES, SQUARES, BUDGET);
    let mut kind_index = 0;
    // `None` while placing units; `Some` once the battle has started.
    let mut battle: Option<Battle> = None;
    let mut running = false;
    let mut timer = 0.;
    loop {
        let layout = compute_layout();

        let status = match battle.as_mut() {
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
                if is_key_pressed(KeyCode::Space) {
                    battle = Some(Battle::new(setup.board().clone()));
                    running = false;
                    timer = 0.;
                }
                placing_status(&setup, kind)
            }
            Some(b) => {
                if is_key_pressed(KeyCode::Space) {
                    running = !running;
                }
                if is_key_pressed(KeyCode::N) {
                    // Stepping by hand pauses auto-play.
                    running = false;
                    b.round();
                }
                if running {
                    timer += get_frame_time();
                    while timer >= ROUND_SECONDS {
                        timer -= ROUND_SECONDS;
                        b.round();
                    }
                }
                if b.outcome() != Outcome::Ongoing {
                    running = false;
                }
                let lines = battle_status(b, running);
                if is_key_pressed(KeyCode::R) {
                    battle = None;
                }
                lines
            }
        };

        clear_background(BACKGROUND);
        draw_grid(&layout);
        match &battle {
            Some(b) => draw_units(b.board(), &layout),
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
    fn cell_at_rejects_outside_the_board() {
        assert_eq!(cell_at(&layout(), 99., 60.), None);
        assert_eq!(cell_at(&layout(), 120., 49.), None);
        assert_eq!(cell_at(&layout(), 420., 60.), None);
        assert_eq!(cell_at(&layout(), 120., 370.), None);
    }
}
