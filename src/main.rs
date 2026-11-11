use macroquad::prelude::*;

const SQUARES: usize = 32;
const PAD: f32 = 10.;
const THICKNESS: f32 = 2.;

struct Layout {
    offset_x: f32,
    offset_y: f32,
    board_size: f32,
    sq_size: f32,
}

fn compute_layout() -> Layout {
    let squares = SQUARES as f32;
    let game_size = screen_width().min(screen_height());
    let board_size = game_size - 2. * PAD;
    let offset_x = (screen_width() - game_size) / 2. + PAD;
    let offset_y = (screen_height() - game_size) / 2. + PAD;
    let sq_size = board_size / squares;
    Layout {
        offset_x,
        offset_y,
        board_size,
        sq_size,
    }
}

fn draw_grid(layout: &Layout) {
    draw_rectangle(
        layout.offset_x,
        layout.offset_y,
        layout.board_size,
        layout.board_size,
        WHITE,
    );

    // Horizontal grid lines
    for i in 1..SQUARES {
        let line = layout.sq_size * i as f32;
        draw_line(
            layout.offset_x,
            layout.offset_y + line,
            layout.offset_x + layout.board_size,
            layout.offset_y + line,
            THICKNESS,
            BLACK,
        );
    }

    // Vertical grid lines
    for i in 1..SQUARES {
        let line = layout.sq_size * i as f32;
        draw_line(
            layout.offset_x + line,
            layout.offset_y,
            layout.offset_x + line,
            layout.offset_y + layout.board_size,
            THICKNESS,
            BLACK,
        );
    }
}

#[macroquad::main("War Automata")]
async fn main() {
    loop {
        let layout = compute_layout();
        draw_grid(&layout);
        next_frame().await;
    }
}
