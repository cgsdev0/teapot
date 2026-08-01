#![allow(clippy::collapsible_if)]
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    hpgl: bool,
}
use imgui::{Context, FontSource};
use raylib_imgui_rs::Renderer;

pub mod app;
pub mod bounding_box;
pub mod geometry;
pub mod navigator;
pub mod renderer;

use crate::{
    app::*,
    renderer::{HpglRenderer, RaylibRenderer},
};

fn main() {
    let mut app = AppState::new();
    app.restart();
    let args = Args::parse();
    // app.nav.zoom.add_padding(100.0, 100.0);
    let (mut rl, thread) = raylib::init()
        .size(1030, 765)
        .title("TeaPlot")
        .log_level(raylib::ffi::TraceLogLevel::LOG_NONE)
        .build();

    let mut imgui = Context::create();
    imgui
        .fonts()
        .add_font(&[FontSource::DefaultFontData { config: None }]);

    let mut renderer = Renderer::create(&mut imgui, &mut rl, &thread);

    while !rl.window_should_close() {
        renderer.update(&mut imgui, &mut rl);
        if !imgui.io().want_capture_mouse {
            app.update(&mut rl, &mut imgui);
        }
        let d = rl.begin_drawing(&thread);
        let mut r = RaylibRenderer {
            d,
            zoom: app.nav.zoom.as_bb(),
        };
        app.render(&mut r, &mut imgui);
        renderer.render(&mut imgui, &mut r.d);
    }
}
