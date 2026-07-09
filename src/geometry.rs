use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::i_float::float::compatible::FloatPointCompatible;
use i_triangle::float::triangulatable::Triangulatable;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::array::IntoIter;
use std::ops::{Add, Mul, Sub};

#[derive(Copy, Clone, Debug)]
pub struct Line {
    pub a: Point,
    pub b: Point,
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        (self.a == other.a && self.b == other.b) || (self.b == other.a && self.a == other.b)
    }
}

impl Eq for Line {}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Intersection {
    pub a: Line,
    pub b: Line,
    pub point: Point,
    pub real: bool,
    pub projected: bool,
}

impl Intersection {
    pub fn shares_line_with(&self, other: &Intersection) -> bool {
        self.a == other.a || self.b == other.b || self.a == other.b || self.b == other.a
    }
    pub fn lines(&self) -> IntoIter<Line, 2> {
        [self.a, self.b].into_iter()
    }
    pub fn on_line(&self, line: Line) -> bool {
        self.a == line || self.b == line
    }
    pub fn other_line(&self, line: Line) -> Line {
        if self.a == line {
            self.b
        } else {
            self.a
        }
    }
}

impl Line {
    pub fn points(&self) -> IntoIter<Point, 2> {
        [self.a, self.b].into_iter()
    }
    pub fn other_point(&self, point: &Point) -> Point {
        if self.a == *point {
            self.b
        } else {
            self.a
        }
    }

    pub fn shares_point_with(&self, other: &Self) -> bool {
        self.a == other.a || self.b == other.b || self.b == other.a || self.a == other.b
    }

    pub fn has_point(&self, p: Point) -> bool {
        self.a == p || self.b == p
    }

    pub fn parallel_with(&self, other: &Line) -> bool {
        let slope_a = self.b - self.a;
        let slope_a = slope_a.y / slope_a.x;
        let slope_b = other.b - other.a;
        let slope_b = slope_b.y / slope_b.x;
        let diff = slope_a - slope_b;
        diff.abs() < f64::EPSILON * 5000.0
    }

    pub fn intersection(&self, other: &Line) -> Option<Intersection> {
        if self == other || self.parallel_with(other) {
            None
        } else {
            let a = *self;
            let b = *other;
            let t = ((a.a.x - b.a.x) * (b.a.y - b.b.y) - (a.a.y - b.a.y) * (b.a.x - b.b.x))
                / ((a.a.x - a.b.x) * (b.a.y - b.b.y) - (a.a.y - a.b.y) * (b.a.x - b.b.x));
            let u = -((a.a.x - a.b.x) * (a.a.y - b.a.y) - (a.a.y - a.b.y) * (a.a.x - b.a.x))
                / ((a.a.x - a.b.x) * (b.a.y - b.b.y) - (a.a.y - a.b.y) * (b.a.x - b.b.x));
            Some(Intersection {
                a,
                b,
                point: Point {
                    x: a.a.x + t * (a.b.x - a.a.x),
                    y: a.a.y + t * (a.b.y - a.a.y),
                    // unused
                    z: 0.0,
                },
                real: !a.shares_point_with(&b)
                    && (0.0..=1.0).contains(&t)
                    && (0.0..=1.0).contains(&u),
                projected: !a.shares_point_with(&b)
                    && (0.0..=1.0).contains(&t)
                    && !(0.0..=1.0).contains(&u),
            })
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
}

impl Triangle {
    pub fn points(&self) -> IntoIter<Point, 3> {
        [self.a, self.b, self.c].into_iter()
    }
    pub fn lines(&self) -> IntoIter<Line, 3> {
        [
            Line {
                a: self.a,
                b: self.b,
            },
            Line {
                a: self.b,
                b: self.c,
            },
            Line {
                a: self.c,
                b: self.a,
            },
        ]
        .into_iter()
    }
    pub fn my_lines(&self, point: &Point) -> Vec<Line> {
        self.lines().filter(|l| l.has_point(*point)).collect()
    }
    pub fn other_line(&self, line: &Line, point: &Point) -> Line {
        self.lines()
            .find(|l| l != line && l.has_point(*point))
            .unwrap()
    }
    pub fn intersections(&self, other: &Triangle) -> Vec<Intersection> {
        let mut result = vec![];
        for l1 in self.lines() {
            for l2 in other.lines() {
                match l1.intersection(&l2) {
                    None => {}
                    Some(i) => {
                        result.push(i);
                    }
                }
            }
        }
        result
    }
    pub fn contains(&self, point: &Point) -> bool {
        if *point == self.a || *point == self.b || *point == self.c {
            return false;
        }
        let det = (self.b.x - self.a.x) * (self.c.y - self.a.y)
            - (self.b.y - self.a.y) * (self.c.x - self.a.x);

        det * ((self.b.x - self.a.x) * (point.y - self.a.y)
            - (self.b.y - self.a.y) * (point.x - self.a.x))
            >= 0.0
            && det
                * ((self.c.x - self.b.x) * (point.y - self.b.y)
                    - (self.c.y - self.b.y) * (point.x - self.b.x))
                >= 0.0
            && det
                * ((self.a.x - self.c.x) * (point.y - self.c.y)
                    - (self.a.y - self.c.y) * (point.x - self.c.x))
                >= 0.0
    }
    pub fn area(&self) -> f64 {
        let a = self.a.x * (self.b.y - self.c.y)
            + self.b.x * (self.c.y - self.a.y)
            + self.c.x * (self.a.y - self.b.y);
        a.abs() * 0.5
    }
}

impl PartialEq for Triangle {
    fn eq(&self, other: &Self) -> bool {
        self.lines().all(|a| other.lines().any(|b| a == b))
    }
}

#[derive(Debug)]
pub struct ConvexPolygon(Vec<Point>);

impl ConvexPolygon {
    pub fn triangulate(&self) -> Vec<Triangle> {
        let mut result = vec![];
        for i in 0..self.0.len() - 2 {
            result.push(Triangle {
                a: self.0[0],
                b: self.0[i + 1],
                c: self.0[i + 2],
            });
        }
        result
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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
    // fn draw(&self) {
    //     let (x1, y1) = canvas(self.min);
    //     let (x2, y2) = canvas(self.max);
    //     println!(
    //         "ctx.strokeStyle='cyan';ctx.strokeRect({}, {}, {}, {});",
    //         x1,
    //         y1,
    //         x2 - x1,
    //         y2 - y1
    //     );
    // }
}

impl Eq for Triangle {}

impl Sub for Triangle {
    type Output = Vec<Triangle>;
    fn sub(self, other: Triangle) -> Vec<Triangle> {
        let subj = [self.a, self.b, self.c];

        let clip = [other.a, other.b, other.c];
        let shapes = subj
            .overlay(&clip, OverlayRule::Difference, FillRule::EvenOdd)
            .iter()
            .map(|shape| shape.triangulate_as::<i64>().to_triangulation::<usize>())
            .collect::<Vec<_>>();

        let mut result = vec![];
        for shape in shapes {
            let bleh: Vec<_> = shape.indices.iter().map(|&i| shape.points[i]).collect();
            for points in bleh.chunks_exact(3) {
                let [a, b, c] = points else { unreachable!() };
                result.push(Triangle {
                    a: *a,
                    b: *b,
                    c: *c,
                });
            }
        }

        result
            .into_iter()
            .filter(|t| t.area() > f64::EPSILON * 10000000.0)
            .collect()
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        let x = OrderedFloat::from(self.x);
        let y = OrderedFloat::from(self.y);
        let z = OrderedFloat::from(self.z);
        let ox = OrderedFloat::from(other.x);
        let oy = OrderedFloat::from(other.y);
        let oz = OrderedFloat::from(other.z);
        x == ox && y == oy && z == oz
    }
}

impl Eq for Point {}

impl From<Point> for raylib::ffi::Vector2 {
    fn from(item: Point) -> Self {
        raylib::ffi::Vector2 {
            x: item.x as f32,
            y: item.y as f32,
        }
    }
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the write! macro to format your output into the buffer `f`
        write!(f, "({:.5}, {:.5})", self.x, self.y)
    }
}

impl FloatPointCompatible for Point {
    type Scalar = f64;

    fn from_xy(x: f64, y: f64) -> Self {
        Self { x, y, z: 0.0 }
    }

    fn x(&self) -> f64 {
        self.x
    }

    fn y(&self) -> f64 {
        self.y
    }
}

impl Point {
    pub fn normalize(&self) -> Point {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Point {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    pub fn dot(&self, other: &Point) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn dist2(&self, other: &Point) -> f64 {
        let x = other.x - self.x;
        let y = other.y - self.y;
        let z = other.z - self.z;
        x * x + y * y + z * z
    }

    pub fn dist(&self, other: &Point) -> f64 {
        self.dist2(other).sqrt()
    }

    pub fn closest_intersection(
        &self,
        intersections: &Vec<&Intersection>,
        line: Line,
    ) -> Intersection {
        intersections
            .iter()
            .filter(|i| i.on_line(line))
            .fold(
                (None, f64::INFINITY),
                |(mut best, mut best_dist): (Option<Intersection>, f64), &intersection| {
                    let d = intersection.point.dist2(self);
                    if d < best_dist {
                        best = Some(*intersection);
                        best_dist = d;
                    }
                    (best, best_dist)
                },
            )
            .0
            .unwrap()
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let x = OrderedFloat::from(self.x);
        let y = OrderedFloat::from(self.y);
        let ox = OrderedFloat::from(other.x);
        let oy = OrderedFloat::from(other.y);
        x.cmp(&ox).then(y.cmp(&oy))
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
impl Mul<f64> for Point {
    type Output = Point;
    fn mul(self, s: f64) -> Point {
        Point {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}
