use macroquad::prelude::*;

const SQUARES: i16 = 32;
const PAD: f32 = 10.;
const THICKNESS: f32 = 2.;

#[macroquad::main("War Automata")]
async fn main() {
    loop {
        let game_size = screen_width().min(screen_height());
        let board_size = game_size - 2. * PAD;
        let offset_x = (screen_width() - game_size) / 2. + PAD;
        let offset_y = (screen_height() - game_size) / 2. + PAD;
        let sq_size = board_size / SQUARES as f32;

        draw_rectangle(offset_x, offset_y, board_size, board_size, WHITE);

        // Horizontal grid lines
        for i in 1..SQUARES {
            draw_line(
                offset_x,
                offset_y + sq_size * i as f32,
                offset_x + board_size,
                offset_y + sq_size * i as f32,
                THICKNESS,
                BLACK,
            );
        }

        // Vertical grid lines
        for i in 1..SQUARES {
            draw_line(
                offset_x + sq_size * i as f32,
                offset_y,
                offset_x + sq_size * i as f32,
                offset_y + board_size,
                THICKNESS,
                BLACK,
            );
        }

        next_frame().await;
    }
}
