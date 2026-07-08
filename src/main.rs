use raylib::prelude::*;

mod app;
mod geometry;

use crate::app::*;
use crate::geometry::*;

fn main() {
    let (mut rl, thread) = raylib::init().size(1030, 765).title("Hello, World").build();
    let app = AppState::new();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);
        d.draw_text("Hello, world!", 12, 12, 20, Color::WHITE);
    }
}
