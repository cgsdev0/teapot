use std::fs;

use crate::geometry::{BBMode, BoundingBox, Point};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum SliceThing {
    OneFace(usize),
    TwoFace(usize, usize),
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Copy, Clone, Debug)]
pub enum AppView {
    Main,
    Painter { face: usize },
    SliceView { face: SliceThing, idx: usize },
    NotFound,
}

#[derive(Serialize, Deserialize)]
pub struct Navigator {
    stack: Vec<AppView>,
    redo: Vec<AppView>,
    pub clip: bool,
    pub zoom: crate::geometry::BoundingBox,
}

impl Navigator {
    pub fn new() -> Self {
        let from_disk = fs::read_to_string("nav_state.json")
            .map_err(|_| ())
            .and_then(|s| serde_json::from_str::<Navigator>(&s).map_err(|_| ()));
        match from_disk {
            Ok(data) => data,
            Err(_) => {
                println!("no 'nav_state.json' file found");
                let mut me = Navigator {
                    stack: vec![],
                    redo: vec![],
                    zoom: BoundingBox::new(),
                    clip: true,
                };
                me.reset_zoom();
                me
            }
        }
    }

    pub fn save(&self) {
        let res = fs::write("nav_state.json", serde_json::to_string(&self).unwrap());
        if let Err(err) = res {
            eprintln!("failed to save file: {:?}", err);
        }
    }

    pub fn zoom_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let min = self.zoom.unproject(&Point {
            x: x1.min(x2).into(),
            y: y1.min(y2).into(),
            z: 0.0.into(),
        });
        let max = self.zoom.unproject(&Point {
            x: x2.max(x1).into(),
            y: y2.max(y1).into(),
            z: 0.0.into(),
        });
        self.zoom = BoundingBox {
            min,
            max,
            mode: BBMode::FromTopLeft,
        };
        self.save();
    }

    pub fn reset_zoom(&mut self) {
        self.zoom = BoundingBox::new();
        self.zoom.mode = BBMode::FromTopLeft;
        self.zoom.expand(&Point {
            x: 0.0.into(),
            y: 0.0.into(),
            z: 0.0.into(),
        });
        self.zoom.expand(&Point {
            x: 1030.0.into(),
            y: 765.0.into(),
            z: 0.0.into(),
        });
        self.save();
    }

    pub fn current(&self) -> AppView {
        self.stack
            .iter()
            .next_back()
            .copied()
            .unwrap_or(AppView::Main)
    }

    pub fn push(&mut self, view: AppView) {
        self.stack.push(view);
        // prevent the stack from getting too beeg
        if self.stack.len() > 100 {
            self.stack.drain(0..20);
        }
        self.save();
    }

    pub fn go_back(&mut self) {
        let last = self.stack.pop();
        if let Some(last) = last {
            self.redo.push(last);
            // prevent the redo from getting too beeg
            if self.redo.len() > 100 {
                self.redo.drain(0..20);
            }
        }
        self.save();
    }

    pub fn go_forward(&mut self) {
        let last = self.redo.pop();
        if let Some(last) = last {
            self.push(last);
        }
        self.save();
    }

    pub fn replace(&mut self, view: AppView) {
        self.stack.pop();
        self.stack.push(view);
        self.save();
    }

    pub fn pop(&mut self) -> Option<AppView> {
        let res = self.stack.pop();
        self.save();
        res
    }
}
