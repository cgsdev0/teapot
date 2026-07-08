use raylib::prelude::*;

mod app;
mod geometry;

use crate::app::*;
use crate::geometry::*;

fn main() {
    let (mut rl, thread) = raylib::init().size(1030, 765).title("Hello, World").build();
    let mut app = AppState::new();
    app.restart();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        app.render(&mut d);
    }
}
