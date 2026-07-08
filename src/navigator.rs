use crate::geometry::{BBMode, BoundingBox, Point};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct NavState {
    pub route: AppView,
    pub clip: bool,
    pub zoom: crate::geometry::BoundingBox,
}

impl NavState {
    pub fn new() -> Self {
        let mut zoom = BoundingBox::new();
        zoom.mode = BBMode::FromTopLeft;
        zoom.expand(&Point {
            x: 0.0.into(),
            y: 0.0.into(),
            z: 0.0.into(),
        });
        zoom.expand(&Point {
            x: 1030.0.into(),
            y: 765.0.into(),
            z: 0.0.into(),
        });
        NavState {
            route: AppView::Main,
            clip: false,
            zoom,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct Navigator {
    stack: Vec<NavState>,
}

impl Navigator {
    pub fn new() -> Self {
        Navigator { stack: vec![] }
    }
    pub fn current(&self) -> NavState {
        self.stack
            .iter()
            .next_back()
            .copied()
            .unwrap_or(NavState::new())
    }
}
