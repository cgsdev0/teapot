use crate::geometry::*;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BBMode {
    FromCenter,
    FromTopLeft,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
    pub mode: BBMode,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundingBox {
    pub fn new() -> Self {
        Self {
            min: Point {
                x: f64::INFINITY,
                y: f64::INFINITY,
                z: 0.0,
            },
            max: Point {
                x: f64::NEG_INFINITY,
                y: f64::NEG_INFINITY,
                z: 0.0,
            },
            mode: BBMode::FromCenter,
        }
    }
    pub fn make_square(&mut self) {
        let dx = self.max.x - self.min.x;
        let dy = self.max.y - self.min.y;
        if dy > dx {
            let delta = dy - dx;
            self.min.x -= delta * 0.5;
            self.max.x += delta * 0.5;
        } else if dx > dy {
            let delta = dx - dy;
            self.min.y -= delta * 0.5;
            self.max.y += delta * 0.5;
        }
    }

    pub fn expand(&mut self, point: &Point) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    pub fn reproject1d(&self, p: f64, start: f64, end: f64, dim: f64) -> f64 {
        match self.mode {
            BBMode::FromCenter => (p - start) / (end - start) * 2.0 - 1.0,
            BBMode::FromTopLeft => (p - start) * (dim / (end - start)),
        }
    }

    pub fn unproject1d(&self, p: f64, start: f64, end: f64, dim: f64) -> f64 {
        match self.mode {
            BBMode::FromCenter => (p + 1.0) / 2.0 * (end - start) + start,
            BBMode::FromTopLeft => p / dim * (end - start) + start,
        }
    }
    // re-project a point zoomed to fit the bounding box
    pub fn reproject(&self, point: &Point) -> Point {
        // let zx = (x) => (x - box.x1) * (canvas.width / (box.x2 - box.x1));
        // let zy = (y) => (y - box.y1) * (canvas.height / (box.y2 - box.y1));
        let xrange = self.reproject1d(point.x, self.min.x, self.max.x, 1030.0);
        let yrange = self.reproject1d(point.y, self.min.y, self.max.y, 765.0);
        Point {
            x: xrange,
            y: yrange,
            z: 0.0,
        }
    }

    pub fn unproject(&self, point: &Point) -> Point {
        let xrange = self.unproject1d(point.x, self.min.x, self.max.x, 1030.0);
        let yrange = self.unproject1d(point.y, self.min.y, self.max.y, 765.0);
        Point {
            x: xrange,
            y: yrange,
            z: 0.0,
        }
    }

    pub fn reproject_bb(&self, bb: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: self.reproject(&bb.min),
            max: self.reproject(&bb.max),
            mode: bb.mode,
        }
    }

    pub fn reproject_line(&self, edge: &Line) -> Line {
        Line {
            a: self.reproject(&edge.a),
            b: self.reproject(&edge.b),
        }
    }
    pub fn reproject_triangle(&self, tri: &Triangle) -> Triangle {
        Triangle {
            a: self.reproject(&tri.a),
            b: self.reproject(&tri.b),
            c: self.reproject(&tri.c),
        }
    }

    pub fn add_padding(&mut self, h: f64, v: f64) {
        self.min.x -= h / 2.0;
        self.min.y -= v / 2.0;
        self.max.x += h / 2.0;
        self.max.y += v / 2.0;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AnimatedBoundingBox {
    time: f64,
    target: Option<BoundingBox>,
    current: BoundingBox,
}

impl AnimatedBoundingBox {
    pub fn new(bb: BoundingBox) -> Self {
        AnimatedBoundingBox {
            time: 0.0,
            target: None,
            current: bb,
        }
    }

    pub fn set_target(&mut self, bb: BoundingBox) {
        if bb.mode != self.current.mode {
            panic!("can't animate between bounding boxes with different modes");
        }
        self.time = 0.0;
        self.target = Some(bb);
    }

    pub fn update(&mut self, delta_time: f64) {
        let Some(target) = self.target else {
            return;
        };
        self.time += delta_time * 2.0;
        if self.time > 1.0 {
            self.current = target;
            self.target = None;
        }
    }

    fn ease_out(x: f64) -> f64 {
        1.0 - (1.0 - x) * (1.0 - x)
    }

    pub fn as_bb(&self) -> BoundingBox {
        let Some(target) = self.target else {
            return self.current;
        };
        let time = Self::ease_out(self.time);
        let aw = self.current.max.x - self.current.min.x;
        let bw = target.max.x - target.min.x;
        let ah = self.current.max.y - self.current.min.y;
        let bh = target.max.y - target.min.y;
        let ax = self.current.min.x;
        let bx = target.min.x;
        let ay = self.current.min.y;
        let by = target.min.y;
        let w = aw + (bw - aw) * time;
        let h = ah + (bh - ah) * time;
        let x = ax + (bx - ax) * time;
        let y = ay + (by - ay) * time;
        BoundingBox {
            mode: self.current.mode,
            min: Point { x, y, z: 0.0 },
            max: Point {
                x: x + w,
                y: y + h,
                z: 0.0,
            },
        }
    }
}
