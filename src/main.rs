use clap::Parser;
use raylib::prelude::RaylibDrawHandle;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    hpgl: bool,
}

pub mod app;
pub mod geometry;
pub mod navigator;
pub mod renderer;

use crate::{app::*, renderer::RaylibRenderer};

fn main() {
    let mut app = AppState::new();
    app.restart();
    let args = Args::parse();
    app.nav.zoom.add_padding(100.0, 100.0);
    let (mut rl, thread) = raylib::init().size(1030, 765).title("Teaplot").build();

    while !rl.window_should_close() {
        app.update(&mut rl);
        let d = rl.begin_drawing(&thread);
        let mut r = RaylibRenderer { d };
        app.render(&mut r);
    }
}
