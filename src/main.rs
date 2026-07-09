use raylib::prelude::*;

pub mod app;
pub mod geometry;
pub mod navigator;

use crate::app::*;

fn main() {
    let (mut rl, thread) = raylib::init().size(1030, 765).title("Hello, World").build();
    let mut app = AppState::new();
    app.restart();

    while !rl.window_should_close() {
        app.update(&mut rl);
        let mut d = rl.begin_drawing(&thread);
        app.render(&mut d);
    }
}
