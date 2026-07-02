// based on the video https://www.youtube.com/watch?v=qjWkNZ0SXfo
// by tsoding :D

use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::{Add, Mul, Sub};

use ordered_float::OrderedFloat;

type F64 = OrderedFloat<f64>;

#[allow(dead_code)]
fn draw_line_js(p1: Point, p2: Point) {
    let (x, y) = canvas(p1);
    println!("ctx.moveTo({}, {});", x, y);
    let (x, y) = canvas(p2);
    println!("ctx.lineTo({}, {});", x, y);
}

use std::sync::atomic::{AtomicBool, Ordering};

static DONT_RECURSE: AtomicBool = AtomicBool::new(false);

#[allow(dead_code)]
fn cursed_subtraction_debug(t: &Triangle, cutter: &Triangle, i: &Vec<Intersection>) {
    if DONT_RECURSE.load(Ordering::Relaxed) {
        return;
    }
    DONT_RECURSE.store(true, Ordering::Relaxed);
    let mut bb = BoundingBox::new();
    bb.expand(&t.a);
    bb.expand(&t.b);
    bb.expand(&t.c);
    bb.expand(&cutter.a);
    bb.expand(&cutter.b);
    bb.expand(&cutter.c);
    bb.draw();
    let t2 = bb.reproject_triangle(&t);
    let c2 = bb.reproject_triangle(&cutter);
    draw_triangle_js(&t2, Color::Lhs);
    draw_triangle_js(&c2, Color::Rhs);
    let mut points: HashMap<Point, usize> = std::collections::HashMap::new();
    for point in vec![t2.a, t2.b, t2.c, c2.a, c2.b, c2.c] {
        match points.get(&point) {
            Some(count) => {
                points.insert(point, count + 1);
            }
            None => {
                points.insert(point, 1);
            }
        }
    }
    for (point, count) in points.iter() {
        draw_point_js(
            *point,
            match count {
                1 => "magenta",
                _ => "yellow",
            },
            false,
        );
    }
    for aye in i {
        let eye = bb.reproject(&aye.point);
        if !aye.projected && !aye.real {
            continue;
        }
        draw_point_js(
            eye,
            if aye.projected { "#33aaff" } else { "#fff" },
            !aye.real,
        );
    }
    let split = t2 - c2;
    eprintln!("{} splits", split.len());
    for t in split {
        draw_triangle_js(&t, Color::Difference);
    }
    // eprintln!("{:?}\n\n{:?}\n\n{:?}", bb, t, cutter);
    std::process::exit(0);
}

fn draw_point_js(point: Point, color: &str, open: bool) {
    println!("ctx.beginPath();");
    println!("ctx.fillStyle = '{}';", color);
    println!("ctx.strokeStyle = '{}';", color);
    let (x, y) = canvas(t2(point));
    println!("ctx.arc({}, {}, 3, 0, 2 * Math.PI);", x, y);
    if open {
        println!("ctx.stroke();");
    } else {
        println!("ctx.fill();");
    }
}
#[allow(dead_code)]
fn draw_triangle_js(t: &Triangle, color: Color) {
    match color {
        Color::Lime => {
            println!("ctx.strokeStyle = 'lime';");
        }
        Color::Red => {
            println!("ctx.strokeStyle = 'red';");
        }
        Color::Lhs => {
            println!("ctx.strokeStyle = '#666';");
        }
        Color::Rhs => {
            println!("ctx.fillStyle = '#ff000030';");
            println!("ctx.strokeStyle = 'red';");
        }
        Color::Difference => {
            println!("ctx.fillStyle = 'transparent';");
            println!("ctx.strokeStyle = 'blue';");
        }
    }
    println!("ctx.beginPath();");
    let (x, y) = canvas(t2(t.a));
    println!("ctx.moveTo({}, {});", x, y);
    let (x, y) = canvas(t2(t.b));
    println!("ctx.lineTo({}, {});", x, y);
    let (x, y) = canvas(t2(t.c));
    println!("ctx.lineTo({}, {});", x, y);
    let (x, y) = canvas(t2(t.a));
    println!("ctx.lineTo({}, {});", x, y);
    println!("ctx.fill();");
    println!("ctx.stroke();");
}

#[allow(dead_code)]
fn draw_line_paper(p1: Point, p2: Point) {
    let (x, y) = paper(p1);
    println!("PU{},{};", x, y);
    let (x, y) = paper(p2);
    println!("PD{},{};", x, y);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct FacePart {
    vertex: Point,
    normal: Point,
}

#[derive(Copy, Clone, Debug)]
struct Line {
    a: Point,
    b: Point,
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        (self.a == other.a && self.b == other.b) || (self.b == other.a && self.a == other.b)
    }
}

impl Eq for Line {}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Intersection {
    a: Line,
    b: Line,
    point: Point,
    real: bool,
    projected: bool,
}

impl Intersection {
    fn on_line(&self, line: Line) -> bool {
        self.a == line || self.b == line
    }
    fn other_line(&self, line: Line) -> Line {
        if self.a == line {
            self.b
        } else {
            self.a
        }
    }
}

impl Line {
    fn points(&self) -> impl Iterator<Item = Point> {
        [self.a, self.b].into_iter()
    }

    fn shares_point_with(&self, other: &Self) -> bool {
        self.a == other.a || self.b == other.b || self.b == other.a || self.a == other.b
    }

    fn has_point(&self, p: Point) -> bool {
        self.a == p || self.b == p
    }

    fn intersection(&self, other: &Line) -> Option<Intersection> {
        if self == other {
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
                    z: 0.0.into(),
                },
                real: !a.shares_point_with(&b)
                    && (0.0..=1.0).contains(&t.into())
                    && (0.0..=1.0).contains(&u.into()),
                projected: !a.shares_point_with(&b)
                    && (0.0..=1.0).contains(&t.into())
                    && !(0.0..=1.0).contains(&u.into()),
            })
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Color {
    Lime,
    Red,
    Lhs,
    Rhs,
    Difference,
}

#[derive(Copy, Clone, Debug)]
struct Triangle {
    a: Point,
    b: Point,
    c: Point,
}

impl Triangle {
    fn points(&self) -> impl Iterator<Item = Point> {
        return [self.a, self.b, self.c].into_iter();
    }
    fn lines(&self) -> Vec<Line> {
        return vec![
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
        ];
    }
    fn intersections(&self, other: &Triangle) -> Vec<Intersection> {
        let mut result = vec![];
        let sl = self.lines();
        let ol = other.lines();
        for l1 in sl.iter() {
            for l2 in ol.iter() {
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
    fn contains(&self, point: &Point) -> bool {
        if *point == self.a || *point == self.b || *point == self.c {
            return false;
        }
        let det = (self.b.x - self.a.x) * (self.c.y - self.a.y)
            - (self.b.y - self.a.y) * (self.c.x - self.a.x);

        det * ((self.b.x - self.a.x) * (point.y - self.a.y)
            - (self.b.y - self.a.y) * (point.x - self.a.x))
            >= 0.0.into()
            && det
                * ((self.c.x - self.b.x) * (point.y - self.b.y)
                    - (self.c.y - self.b.y) * (point.x - self.b.x))
                >= 0.0.into()
            && det
                * ((self.a.x - self.c.x) * (point.y - self.c.y)
                    - (self.a.y - self.c.y) * (point.x - self.c.x))
                >= 0.0.into()
    }
    fn area(&self) -> f64 {
        let a = self.a.x * (self.b.y - self.c.y)
            + self.b.x * (self.c.y - self.a.y)
            + self.c.x * (self.a.y - self.b.y);
        a.abs() * 0.5
    }
}

impl PartialEq for Triangle {
    fn eq(&self, other: &Self) -> bool {
        let l1 = self.lines();
        let l2 = other.lines();
        l1.iter().all(|a| l2.iter().any(|b| a == b))
    }
}

#[derive(Debug)]
struct ConvexPolygon(Vec<Point>);
impl ConvexPolygon {
    fn triangulate(&self) -> Vec<Triangle> {
        let mut result = vec![];
        for i in 0..self.0.len() - 2 {
            result.push(Triangle {
                a: self.0[0],
                b: self.0[i + 1],
                c: self.0[i + 2],
            });
        }
        eprintln!(
            "Turned {} points into {} triangles",
            self.0.len(),
            result.len()
        );
        eprintln!("{:?}\n", result);
        result
    }
}

#[derive(Debug)]
struct BoundingBox {
    min: Point,
    max: Point,
}

impl BoundingBox {
    fn new() -> Self {
        Self {
            min: Point {
                x: f64::INFINITY.into(),
                y: f64::INFINITY.into(),
                z: 0.0.into(),
            },
            max: Point {
                x: f64::NEG_INFINITY.into(),
                y: f64::NEG_INFINITY.into(),
                z: 0.0.into(),
            },
        }
    }

    fn expand(&mut self, point: &Point) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    // re-project a point zoomed to fit the bounding box
    fn reproject(&self, point: &Point) -> Point {
        let xrange = (point.x - self.min.x) / (self.max.x - self.min.x) * 2.0 - 1.0;
        let yrange = (point.y - self.min.y) / (self.max.y - self.min.y) * 2.0 - 1.0;
        un_t2(Point {
            x: xrange * 0.9,
            y: yrange * 0.9,
            z: 0.0.into(),
        })
    }
    fn reproject_triangle(&self, tri: &Triangle) -> Triangle {
        Triangle {
            a: self.reproject(&tri.a),
            b: self.reproject(&tri.b),
            c: self.reproject(&tri.c),
        }
    }
    fn draw(&self) {
        let (x1, y1) = canvas(t2(self.min));
        let (x2, y2) = canvas(t2(self.max));
        println!(
            "ctx.strokeStyle='cyan';ctx.strokeRect({}, {}, {}, {});",
            x1,
            y1,
            x2 - x1,
            y2 - y1
        );
    }
}

impl Eq for Triangle {}

impl Sub for Triangle {
    type Output = Vec<Triangle>;
    fn sub(self, other: Triangle) -> Vec<Triangle> {
        let i = self.intersections(&other);
        let real = i.iter().filter(|i| i.real).collect::<Vec<_>>();
        let imaginary = i.iter().filter(|i| !i.real).collect::<Vec<_>>();
        let projected = i.iter().filter(|i| i.projected).collect::<Vec<_>>();
        let sa = other.contains(&self.a);
        let sb = other.contains(&self.b);
        let sc = other.contains(&self.c);
        let scount = (if sa { 1 } else { 0 }) + (if sb { 1 } else { 0 }) + (if sc { 1 } else { 0 });
        let oa = self.contains(&other.a);
        let ob = self.contains(&other.b);
        let oc = self.contains(&other.c);
        let ocount = (if oa { 1 } else { 0 }) + (if ob { 1 } else { 0 }) + (if oc { 1 } else { 0 });
        let mut polys: Vec<ConvexPolygon> = vec![];
        match (real.len(), projected.len(), scount, ocount) {
            (0, _, _, _) => {
                return vec![self];
            }
            (_, _, 0, 3) => {
                // we are getting a hole bored out of the middle oh no
                cursed_subtraction_debug(&self, &other, &i);
            }
            (_, _, 3, 0) => {
                // we have been fully subtracted
            }
            (6, 0, 0, 0) => {
                for l in self.lines() {
                    let on_this_line = real.iter().filter(|i| i.on_line(l)).collect::<Vec<_>>();
                    if on_this_line.is_empty() {
                        panic!("uh oh");
                    }
                }
            }
            (4, _, 0, 0) => {
                let (lines_with_intersections, lines_without_intersections) = self
                    .lines()
                    .into_iter()
                    .partition::<Vec<_>, _>(|&l| real.iter().any(|&i| i.on_line(l)));
                for l in lines_with_intersections {
                    let on_this_line = real.iter().filter(|i| i.on_line(l)).collect::<Vec<_>>();
                    if on_this_line.is_empty() {
                        panic!("uh oh");
                    }
                    let mut points = vec![l.a];
                    let (next, _): (Option<&Intersection>, F64) = on_this_line.iter().fold(
                        (None, f64::INFINITY.into()),
                        |(mut best, mut best_dist), &intersection| {
                            let d = intersection.point.dist2(&l.a);
                            if d < best_dist {
                                best = Some(intersection);
                                best_dist = d;
                            }
                            (best, best_dist)
                        },
                    );
                    let next = next.unwrap();
                    points.push(next.point);
                    let l = next.other_line(l);
                    let next = real
                        .iter()
                        .find(|i| i.on_line(l) && i.point != next.point)
                        .unwrap();
                    points.push(next.point);
                    let l = next.other_line(l);
                    if l.has_point(points[0]) {
                        // done!
                        polys.push(ConvexPolygon(points));
                    } else {
                        // find and add the one missing point
                        let other_line = lines_without_intersections[0];
                        let other_point = if other_line.a == next.point {
                            other_line.b
                        } else {
                            other_line.a
                        };
                        points.push(other_point);
                        polys.push(ConvexPolygon(points));
                    }
                }
                for poly in &polys {
                    eprintln!("Polygon of degree {}", poly.0.len());
                    eprintln!("Polygon: {:?}", poly.0);
                }
                eprintln!("\n\n\n");
                // cursed_subtraction_debug(&self, &other, &i);
            }
            (4, 0, 1, 0) => {
                for p in self.points().filter(|p| !other.contains(p)) {
                    eprintln!("uncontained point in 4,0,1,0: {:?}", p);
                    let edges = self.lines().into_iter().filter(|l| l.has_point(p));
                    let intersections: Vec<Intersection> = edges.map(|l| {
                        real.iter().filter(|i| i.on_line(l))
                            .fold(
                                (None, f64::INFINITY.into()),
                                |(mut best, mut best_dist): (Option<Intersection>, F64),
                                 &intersection| {
                                    let d = intersection.point.dist2(&p);
                                    if d < best_dist {
                                        best = Some(*intersection);
                                        best_dist = d;
                                    }
                                    (best, best_dist)
                                },
                            )
                            .0
                            .unwrap()
                    }).collect();
                    polys.push(ConvexPolygon(vec![ p, intersections[0].point, intersections[1].point ]));
                }
            }
            (2, _, 1, 0) => {
                polys.push(ConvexPolygon(
                    real.iter()
                        .map(|i| {
                            (if self.lines().into_iter().any(|l| l == i.a) {
                                i.a
                            } else {
                                i.b
                            })
                            .points()
                            .find(|p| !other.contains(&p))
                            .unwrap()
                        })
                        .chain(real.iter().rev().map(|i| i.point))
                        .collect(),
                ));
            }
            (2, _, 2, 0) => {
                let starting_point = self.points().find(|p| !other.contains(p)).unwrap();
                polys.push(ConvexPolygon(vec![
                    starting_point,
                    real[0].point,
                    real[1].point,
                ]));
            }
            (2, 2, 0, 1) => {
                let contained_point = other.points().find(|p| self.contains(p)).unwrap();
                for starting_point in self.points() {
                    let edges: Vec<_> = self
                        .lines()
                        .into_iter()
                        .filter(|l| l.has_point(starting_point))
                        .collect();
                    let intersections: Vec<Intersection> = edges
                        .iter()
                        .map(|&l| {
                            projected
                                .iter()
                                .chain(real.iter())
                                .filter(|i| i.on_line(l))
                                .fold(
                                    (None, f64::INFINITY.into()),
                                    |(mut best, mut best_dist): (Option<Intersection>, F64),
                                     &intersection| {
                                        let d = intersection.point.dist2(&starting_point);
                                        if d < best_dist {
                                            best = Some(*intersection);
                                            best_dist = d;
                                        }
                                        (best, best_dist)
                                    },
                                )
                                .0
                                .unwrap()
                        })
                        .collect();
                    polys.push(ConvexPolygon(vec![
                        starting_point,
                        intersections[0].point,
                        contained_point,
                        intersections[1].point,
                    ]));
                }
                // cursed_subtraction_debug(&self, &other, &i);
            }
            _ => {
                eprintln!("intersections: {}", real.len());
                eprintln!("projected: {}", projected.len());
                eprintln!("scount: {}", scount);
                eprintln!("ocount: {}\n", ocount);
                cursed_subtraction_debug(&self, &other, &i);
            }
        };
        polys
            .iter()
            .flat_map(|poly| poly.triangulate())
            .filter(|tri| {
                let area = tri.area();
                if area > 0.003 {
                    true
                } else {
                    eprintln!("throwing out a triangle of area {}: {:?}", area, tri);
                    false
                }
            })
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Face {
    eyes: FacePart,
    noes: FacePart,
    ears: FacePart,
    hair: Vec<Triangle>,
    culled: bool,
}

// 3. Implement PartialOrd (Required by Ord)
impl PartialOrd for Face {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other)) // Simply defer to the total Ord implementation
    }
}

// 4. Implement Ord (The total ordering logic)
impl Ord for Face {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare x first. If they are equal, move to y, then z.
        self.calc_centroid().z.total_cmp(&other.calc_centroid().z)
    }
}

impl Face {
    fn calc_centroid(&self) -> Point {
        let a = self.eyes.vertex;
        let b = self.noes.vertex;
        let c = self.ears.vertex;
        Point {
            x: (a.x + b.x + c.x) / 3.0,
            y: (a.y + b.y + c.y) / 3.0,
            z: (a.z + b.z + c.z) / 3.0,
        }
    }
    fn calc_normal(&self) -> Point {
        let a = self.eyes.vertex - self.noes.vertex;
        let b = self.ears.vertex - self.noes.vertex;
        Point {
            x: a.y * b.z - a.z * b.y,
            y: a.z * b.x - a.x * b.z,
            z: a.x * b.y - a.y * b.x,
        }
        .normalize()
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct Point {
    x: F64,
    y: F64,
    z: F64,
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the write! macro to format your output into the buffer `f`
        write!(f, "({:.5}, {:.5})", self.x, self.y)
    }
}

impl Point {
    fn normalize(&self) -> Point {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Point {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    fn dot(&self, other: &Point) -> F64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn dist2(&self, other: &Point) -> F64 {
        let x = other.x - self.x;
        let y = other.y - self.y;
        let z = other.z - self.z;
        x * x + y * y + z * z
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
impl Mul<F64> for Point {
    type Output = Point;
    fn mul(self, s: F64) -> Point {
        Point {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

#[allow(dead_code)]
fn paper(p: Point) -> (u16, u16) {
    (
        ((p.x.into_inner() + 1.0) / 2.0 * 7650.0 + 1000.0) as u16,
        (((-p.y).into_inner() + 1.0) / 2.0 * 7650.0) as u16,
    )
}

#[allow(dead_code)]
fn canvas(p: Point) -> (F64, F64) {
    (
        ((p.x + 1.0) / 2.0 * 765.0 + 100.0),
        ((-p.y + 1.0) / 2.0 * 765.0),
    )
}

fn un_t2(p: Point) -> Point {
    Point {
        x: p.x,
        y: p.y - 0.25,
        z: p.z,
    }
}

fn t2(p: Point) -> Point {
    Point {
        x: p.x,
        y: p.y + 0.25,
        z: p.z,
    }
}

fn project(p: Point) -> Point {
    Point {
        x: p.x / (p.z / 2.0),
        y: p.y / (p.z / 2.0),
        z: 0.0.into(),
    }
}

fn translate(p: Point) -> Point {
    Point {
        x: p.x,
        y: p.y - 0.9,
        z: p.z + 5.0,
    }
}

fn rotate(p: Point, angle: F64) -> Point {
    let c = angle.cos();
    let s = angle.sin();
    Point {
        x: p.x * c - p.z * s,
        z: p.x * s + p.z * c,
        y: p.y,
    }
}

fn main() {
    let contents: String = fs::read_to_string("teapot.obj").unwrap();
    let mut faces: Vec<Face> = vec![];
    let mut v: Vec<Point> = vec![];
    let mut vn: Vec<Point> = vec![];
    let dt: F64 = (std::f64::consts::PI / 2.0).into();
    for line in contents.lines() {
        let parts = line.split(" ").collect::<Vec<_>>();
        match parts[0] {
            "f" => {
                let parts = parts
                    .iter()
                    .skip(1)
                    .map(|p| {
                        let parts = p.split("/").collect::<Vec<_>>();
                        FacePart {
                            vertex: translate(rotate(
                                v[parts[0].parse::<usize>().unwrap() - 1],
                                dt,
                            )),
                            normal: vn[parts[2].parse::<usize>().unwrap() - 1],
                        }
                    })
                    .collect::<Vec<_>>();
                faces.push(Face {
                    eyes: parts[0],
                    noes: parts[1],
                    ears: parts[2],
                    hair: vec![Triangle {
                        a: project(parts[0].vertex),
                        b: project(parts[1].vertex),
                        c: project(parts[2].vertex),
                    }],
                    culled: false,
                });
            }
            "v" => {
                v.push(Point {
                    x: parts[1].parse::<F64>().unwrap(),
                    y: parts[2].parse::<F64>().unwrap(),
                    z: parts[3].parse::<F64>().unwrap(),
                });
            }
            "vn" => {
                vn.push(Point {
                    x: parts[1].parse::<F64>().unwrap(),
                    y: parts[2].parse::<F64>().unwrap(),
                    z: parts[3].parse::<F64>().unwrap(),
                });
            }
            _ => {}
        }
    }
    // let frame = 6;

    let mut count = 0;
    faces.sort();

    // backface culling
    for face in faces.iter_mut() {
        let n = face.calc_normal();
        let c = face.calc_centroid().normalize();
        let which_way = n.dot(&c);
        if which_way <= 0.0.into() {
            face.culled = true;
        }
    }
    let mut drawn: Vec<&mut Face> = vec![];
    // this is where it gets hairy
    for face in faces.iter_mut() {
        if face.culled {
            continue;
        }
        // XXX: this is potentially teapot specific
        // this culls triangles whose points are all occluded
        // we might not need it!
        face.hair = face
            .hair
            .clone()
            .into_iter()
            .filter(|t| {
                let mut a = true;
                let mut b = true;
                let mut c = true;
                for f2 in drawn.iter() {
                    for t2 in f2.hair.iter() {
                        if t2.contains(&t.a) {
                            a = false;
                        }
                        if t2.contains(&t.b) {
                            b = false;
                        }
                        if t2.contains(&t.c) {
                            c = false;
                        }
                    }
                }
                a || b || c
            })
            .collect();
        drawn.push(face);
    }
    drop(drawn);

    let mut drawn: Vec<&mut Face> = vec![];
    // it's time to split hairs
    for face in faces.iter_mut() {
        if face.culled {
            continue;
        }
        for f2 in drawn.iter() {
            for t2 in f2.hair.iter() {
                let mut haircut: Vec<Triangle> = vec![];
                for t in face.hair.iter() {
                    let mut split = *t - *t2;
                    haircut.append(&mut split);
                }
                face.hair = haircut;
            }
        }
        drawn.push(face);
    }

    // this is where we do rendering :)
    for face in faces {
        if face.culled {
            continue;
        }
        count += 1;

        for t in face.hair {
            // if t.color == Color::Lime {
            //     continue;
            // }
            draw_triangle_js(&t, Color::Lime);
        }
    }
    println!("console.log('{}');", count);
}
