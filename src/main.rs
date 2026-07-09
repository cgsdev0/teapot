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

use crate::app::*;

fn main() {
    let mut app = AppState::new();
    app.restart();
    let args = Args::parse();
    let mut renderer: Option<&mut RaylibDrawHandle> = None;
    app.nav.zoom.add_padding(100.0, 100.0);
    if args.hpgl {
        app.render(&mut renderer);
    } else {
        let (mut rl, thread) = raylib::init().size(1030, 765).title("Hello, World").build();

        while !rl.window_should_close() {
            app.update(&mut rl);
            let mut d = rl.begin_drawing(&thread);
            renderer = Some(&mut d);
            app.render(&mut renderer);
        }
    }
}
