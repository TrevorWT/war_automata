use macroquad::prelude::*;

const SQUARES: i16 = 32;

#[macroquad::main("War Automata")]
async fn main() {
    loop {
        let game_size = screen_width().min(screen_height());
        let offset_x = (screen_width() - game_size) / 2. + 10.;
        let offset_y = (screen_height() - game_size) / 2. + 10.;
        let sq_size = (screen_height() - offset_y * 2.) / SQUARES as f32;

        draw_rectangle(offset_x, offset_y, game_size - 20., game_size - 20., WHITE);

        // Horizontal grid lines
        for i in 1..SQUARES {
            draw_line(
                offset_x,
                offset_y + sq_size * i as f32,
                screen_width() - offset_x,
                offset_y + sq_size * i as f32,
                2.,
                BLACK,
            );
        }

        // Vertical grid lines
        for i in 1..SQUARES {
            draw_line(
                offset_x + sq_size * i as f32,
                offset_y,
                offset_x + sq_size * i as f32,
                screen_height() - offset_y,
                2.,
                BLACK,
            );
        }

        next_frame().await;
    }
}
