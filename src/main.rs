// based on the video https://www.youtube.com/watch?v=qjWkNZ0SXfo
// by tsoding :D

use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::ops::{Add, Mul, Sub};

use ordered_float::OrderedFloat;

type F64 = OrderedFloat<f64>;

#[allow(dead_code)]
fn draw_line_js(p1: Point, p2: Point) {
    let (x, y) = canvas(p1);
    println!("ctx.moveTo(zx({}), zy({}));", x, y);
    let (x, y) = canvas(p2);
    println!("ctx.lineTo(zx({}), zy({}));", x, y);
}

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static EVIL_MODE: AtomicBool = AtomicBool::new(false);
static DONT_RECURSE: AtomicBool = AtomicBool::new(false);
static DONT_RECURSE2: AtomicBool = AtomicBool::new(false);
static SKIP: AtomicI32 = AtomicI32::new(0);

#[allow(dead_code)]
fn cursed_subtraction_debug(
    t: &Triangle,
    cutter: &Triangle,
    i: &Vec<Intersection>,
    want_panic: bool,
) {
    if !EVIL_MODE.load(Ordering::Relaxed) {
        return;
    }
    let skip = SKIP.load(Ordering::Relaxed);
    if skip < 4 && !want_panic {
        SKIP.store(skip + 1, Ordering::Relaxed);
        return;
    }
    if want_panic {
        if DONT_RECURSE.load(Ordering::Relaxed) {
            return;
        }
        DONT_RECURSE.store(true, Ordering::Relaxed);
    } else {
        if DONT_RECURSE2.load(Ordering::Relaxed) {
            return;
        }
        DONT_RECURSE2.store(true, Ordering::Relaxed);
    }
    eprintln!("cursed entry");
    let mut bb = BoundingBox::new();
    bb.expand(&t.a);
    bb.expand(&t.b);
    bb.expand(&t.c);
    // bb.expand(&cutter.a);
    // bb.expand(&cutter.b);
    // bb.expand(&cutter.c);
    let t2 = bb.reproject_triangle(t);
    let c2 = bb.reproject_triangle(cutter);
    let draw = |tris: Vec<Triangle>| {
        eprintln!("{:?}", t);
        println!("const bounding_boxes = null;");
        println!("const plot = async () => {{");

        bb.draw();
        draw_triangle_js(&t2, Color::Lhs);
        draw_triangle_js(&c2, Color::Rhs);
        let mut points: HashMap<Point, usize> = std::collections::HashMap::new();
        for point in [t2.a, t2.b, t2.c, c2.a, c2.b, c2.c] {
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
        for aye in i
            .iter()
            .filter(|i| i.real && t.points().all(|p| i.point.dist2(&p) > f64::EPSILON.into()))
        {
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
        for t in tris {
            draw_triangle_js(&bb.reproject_triangle(&t), Color::Difference);
        }
        println!("}}");
    };
    if want_panic {
        let res = std::panic::catch_unwind(|| {
            // we don't care about the result;
            // either this panics and we draw,
            // or it doesn't panic and we don't care anyways.
            let _ = *t - *cutter;
        });
        match res {
            Err(_) => {
                eprintln!("PANIC CAPTURED");
                draw(vec![]);
            }
            Ok(_) => {
                // no panic? lame
                DONT_RECURSE.store(false, Ordering::Relaxed);
                return;
            }
        }
    } else {
        let split = *t - *cutter;
        // eprintln!("{} splits", split.len());
        draw(split);
    }
    // eprintln!("{:?}\n\n{:?}\n\n{:?}", bb, t, cutter);
    std::process::exit(0);
}

fn draw_point_js(point: Point, color: &str, open: bool) {
    println!("ctx.beginPath();");
    println!("ctx.fillStyle = '{}';", color);
    println!("ctx.strokeStyle = '{}';", color);
    let (x, y) = canvas(point);
    println!("ctx.arc(zx({}), zy({}), 5, 0, 20 * Math.PI);", x, y);
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
            println!("ctx.strokeStyle = 'transparent';");
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
    let (x, y) = canvas(t.a);
    println!("ctx.moveTo(zx({}), zy({}));", x, y);
    let (x, y) = canvas(t.b);
    println!("ctx.lineTo(zx({}), zy({}));", x, y);
    let (x, y) = canvas(t.c);
    println!("ctx.lineTo(zx({}), zy({}));", x, y);
    let (x, y) = canvas(t.a);
    println!("ctx.lineTo(zx({}), zy({}));", x, y);
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
    fn shares_line_with(&self, other: &Intersection) -> bool {
        self.a == other.a || self.b == other.b || self.a == other.b || self.b == other.a
    }
    fn lines(&self) -> IntoIter<Line, 2> {
        [self.a, self.b].into_iter()
    }
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
    fn points(&self) -> IntoIter<Point, 2> {
        [self.a, self.b].into_iter()
    }
    fn other_point(&self, point: &Point) -> Point {
        if self.a == *point {
            self.b
        } else {
            self.a
        }
    }

    fn shares_point_with(&self, other: &Self) -> bool {
        self.a == other.a || self.b == other.b || self.b == other.a || self.a == other.b
    }

    fn has_point(&self, p: Point) -> bool {
        self.a == p || self.b == p
    }

    fn parallel_with(&self, other: &Line) -> bool {
        let slope_a = self.b - self.a;
        let slope_a = slope_a.y / slope_a.x;
        let slope_b = other.b - other.a;
        let slope_b = slope_b.y / slope_b.x;
        let diff = slope_a - slope_b;
        diff.abs() < f64::EPSILON * 5000.0
    }

    fn intersection(&self, other: &Line) -> Option<Intersection> {
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
    fn to_js_string(&self) -> String {
        let (ax, ay) = canvas(self.a);
        let (bx, by) = canvas(self.b);
        let (cx, cy) = canvas(self.c);
        format!("[[{},{}],[{},{}],[{},{}]]", ax, ay, bx, by, cx, cy)
    }
    fn points(&self) -> IntoIter<Point, 3> {
        [self.a, self.b, self.c].into_iter()
    }
    fn lines(&self) -> IntoIter<Line, 3> {
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
    fn my_lines(&self, point: &Point) -> Vec<Line> {
        self.lines().filter(|l| l.has_point(*point)).collect()
    }
    fn other_line(&self, line: &Line, point: &Point) -> Line {
        self.lines()
            .find(|l| l != line && l.has_point(*point))
            .unwrap()
    }
    fn intersections(&self, other: &Triangle) -> Vec<Intersection> {
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
        self.lines().all(|a| other.lines().any(|b| a == b))
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
        // eprintln!(
        //     "Turned {} points into {} triangles",
        //     self.0.len(),
        //     result.len()
        // );
        // eprintln!("{:?}\n", result);
        result
    }
}

#[derive(Debug)]
struct BoundingBox {
    min: Point,
    max: Point,
}

struct FaceDebug {
    bb: BoundingBox,
    face_id: usize,
    triangle: Triangle,
}

impl FaceDebug {
    fn to_js_string(&self) -> String {
        let (ax, ay) = canvas(self.bb.min);
        let (bx, by) = canvas(self.bb.max);
        format!(
            "{{t: {}, x1: {}, y1: {}, x2: {}, y2: {}, face: {}}},",
            self.triangle.to_js_string(),
            ax,
            by,
            bx,
            ay,
            self.face_id
        )
    }
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
    fn make_square(&mut self) {
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
        Point {
            x: xrange * 0.9,
            y: yrange * 0.9,
            z: 0.0.into(),
        }
    }

    fn reproject_bb(&self, bb: &BoundingBox) -> BoundingBox {
        BoundingBox {
            min: self.reproject(&bb.min),
            max: self.reproject(&bb.max),
        }
    }

    fn reproject_triangle(&self, tri: &Triangle) -> Triangle {
        Triangle {
            a: self.reproject(&tri.a),
            b: self.reproject(&tri.b),
            c: self.reproject(&tri.c),
        }
    }
    fn draw(&self) {
        let (x1, y1) = canvas(self.min);
        let (x2, y2) = canvas(self.max);
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
        let real = i
            .iter()
            .filter(|i| {
                i.real
                    && self
                        .points()
                        .all(|p| i.point.dist2(&p) > f64::EPSILON.into())
            })
            .collect::<Vec<_>>();
        let extras = self.points().filter(|p| {
            i.iter()
                .any(|i| i.real && i.point.dist2(&p) <= f64::EPSILON.into())
        });
        // let real = i.iter().filter(|i| i.real).collect::<Vec<_>>();
        let projected = i.iter().filter(|i| i.projected).collect::<Vec<_>>();
        let mut self_included: HashSet<Point> = std::collections::HashSet::new();
        for point in self.points().filter(|a| other.contains(a)).chain(extras) {
            self_included.insert(point);
        }
        let scount = self_included.iter().count();
        let mut other_included: HashSet<Point> = std::collections::HashSet::new();
        for point in other.points().filter(|a| self.contains(a)) {
            other_included.insert(point);
        }
        let ocount = other_included.iter().count();
        // let ocount = other.points().filter(|a| self.contains(a)).count();
        let mut polys: Vec<ConvexPolygon> = vec![];
        let mut points: HashMap<Point, usize> = std::collections::HashMap::new();
        let mut shared: HashSet<Point> = std::collections::HashSet::new();
        for point in [self.a, self.b, self.c, other.a, other.b, other.c] {
            match points.get(&point) {
                Some(count) => {
                    points.insert(point, count + 1);
                    shared.insert(point);
                }
                None => {
                    points.insert(point, 1);
                }
            }
        }
        // seek the truth
        let shared_count = shared.iter().count();
        if shared_count + scount < 3 && scount > 0 {
            for line in self.lines() {
                let intersections = real.iter().filter(|i| i.on_line(line)).count();
                if self_included.contains(&line.b) {
                    if !self_included.contains(&line.a)
                        && !shared.contains(&line.a)
                        && intersections == 0
                    {
                        self_included.remove(&line.b);
                        shared.insert(line.b);
                    }
                }
                if self_included.contains(&line.a) {
                    if !self_included.contains(&line.b)
                        && !shared.contains(&line.b)
                        && intersections == 0
                    {
                        self_included.remove(&line.a);
                        shared.insert(line.a);
                    }
                }
            }
        }
        let scount = self_included.iter().count();
        let shared_count = shared.iter().count();

        eprintln!("intersections: {}", real.len());
        eprintln!("projected: {}", projected.len());
        eprintln!("scount: {}", scount);
        eprintln!("ocount: {}", ocount);
        eprintln!("shared: {}\n", shared_count);
        // JAGI
        match (real.len(), projected.len(), scount, ocount, shared_count) {
            (_, _, _, _, 3) => {
                // it's the same picture
                return vec![];
            }
            (_, _, 0, 3, _) => {
                // we are getting a hole bored out of the middle oh no
                cursed_subtraction_debug(&self, &other, &i, false);
            }
            (0, 0, 2, 0, 1) | (_, _, 3, 0, _) => {
                // we have been fully subtracted
                return vec![];
            }
            (0, _, 0, _, _) => {
                // no intersections, all good!
                return vec![self];
            }
            // (0, _, 0, _, _)
            (6, 0, 0, 0, _) => {
                // star of david situation
                cursed_subtraction_debug(&self, &other, &i, false);
            }
            (3, _, 0, 0, 1) => {
                let line = self
                    .lines()
                    .find(|line| !real.iter().any(|i| i.on_line(*line)))
                    .unwrap();
                let mut poly_a = ConvexPolygon(vec![line.a, line.b]);
                let other_point = self.points().find(|&p| p != line.a && p != line.b).unwrap();
                let intersections: Vec<_> = self
                    .my_lines(&other_point)
                    .into_iter()
                    .map(|l| {
                        real.iter()
                            .filter(|i| i.on_line(l))
                            .fold(
                                (None, f64::INFINITY.into()),
                                |(mut best, mut best_dist): (Option<Intersection>, F64),
                                 &intersection| {
                                    let d = intersection.point.dist2(&other_point);
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
                    .map(|i| i.point)
                    .collect();
                let x = intersections[0];
                let y = intersections[1];
                let poly_b = ConvexPolygon(vec![other_point, x, y]);
                let z = real
                    .iter()
                    .find(|i| i.point != x && i.point != y)
                    .unwrap()
                    .point;

                poly_a.0.push(z);
                polys.push(poly_a);
                polys.push(poly_b);
            }
            (2, 4, 0, 2, 0) => {
                let double_projected_line = self
                    .lines()
                    .find(|&l| projected.iter().filter(|i| i.on_line(l)).count() > 1)
                    .unwrap();
                let double_intersected_line = self
                    .lines()
                    .find(|&l| real.iter().filter(|i| i.on_line(l)).count() > 1)
                    .unwrap();
                let new_triangle_vertex = self
                    .points()
                    .find(|&p| {
                        double_projected_line.has_point(p) && double_intersected_line.has_point(p)
                    })
                    .unwrap();
                let tri_real =
                    new_triangle_vertex.closest_intersection(&real, double_intersected_line);
                let (tri_cutting_point, concave_cutting_point): (Vec<Point>, Vec<Point>) =
                    other_included
                        .iter()
                        .partition(|&&p| tri_real.b.has_point(p));
                let tri_cutting_point = tri_cutting_point[0];
                let concave_cutting_point = concave_cutting_point[0];

                let tri_proj =
                    new_triangle_vertex.closest_intersection(&projected, double_projected_line);
                polys.push(ConvexPolygon(vec![
                    new_triangle_vertex,
                    tri_real.point,
                    tri_proj.point,
                ]));
                polys.push(ConvexPolygon(vec![
                    concave_cutting_point,
                    tri_cutting_point,
                    tri_proj.point,
                    double_projected_line.other_point(&new_triangle_vertex),
                    double_intersected_line.other_point(&new_triangle_vertex),
                    real.iter()
                        .find(|i| i.point != tri_real.point)
                        .unwrap()
                        .point,
                ]));
            }
            (4, _, 0, 0, _) => {
                let (lines_with_intersections, lines_without_intersections) = self
                    .lines()
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
                    let Some(next) = real.iter().find(|i| i.on_line(l) && i.point != next.point)
                    else {
                        // we tried our best
                        break;
                    };
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
            }
            (4, 0, 1, 0, _) => {
                // cursed_subtraction_debug(&self, &other, &i, true);
                for p in self.points().filter(|p| !self_included.contains(p)) {
                    let edges = self.lines().filter(|l| l.has_point(p));
                    let intersections: Vec<Intersection> = edges
                        .map(|l| {
                            real.iter()
                                .filter(|i| i.on_line(l))
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
                        })
                        .collect();
                    polys.push(ConvexPolygon(vec![
                        p,
                        intersections[0].point,
                        intersections[1].point,
                    ]));
                }
            }
            // (2, _, 0, 0, 1) | (2, _, 1, 0, _) if real[0].shares_line_with(real[1]) => {
            //     panic!("haha");
            // }
            (2, _, 0, 0, 1) if !real[0].shares_line_with(real[1]) => {
                polys.push(ConvexPolygon(
                    real.iter()
                        .map(|i| {
                            (if self.lines().any(|l| l == i.a) {
                                i.a
                            } else {
                                i.b
                            })
                            .points()
                            .find(|p| !self_included.contains(p) && !shared.contains(p))
                            .unwrap()
                        })
                        .chain(real.iter().rev().map(|i| i.point))
                        .collect(),
                ));
            }
            (2, _, 0, 0, 1) => {
                cursed_subtraction_debug(&self, &other, &i, false);
                let start = shared.iter().next().unwrap();
                for p in self.points().filter(|p| p != start) {
                    let closest_intersection = real
                        .iter()
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
                        .unwrap();
                    polys.push(ConvexPolygon(vec![*start, p, closest_intersection.point]));
                }
                /*
                let others: Vec<_> = self.points().filter(|p| p != start).collect();
                let a = others[0];
                let b = others[1];
                polys.push(ConvexPolygon(vec![*start, a]));
                polys.push(ConvexPolygon(vec![*start, b]));
                */
            }
            // (2, _, 0, 2, 0) => {
            //     polys.push(ConvexPolygon(
            //         real.iter()
            //             .map(|i| {
            //                 (if self.lines().any(|l| l == i.a) {
            //                     i.a
            //                 } else {
            //                     i.b
            //                 })
            //                 .points()
            //                 .find(|p| !self_included.contains(p) && !shared.contains(p))
            //                 .unwrap()
            //             })
            //             .chain(real.iter().rev().map(|i| i.point))
            //             .collect(),
            //     ));
            // }
            (2, _, 1, 0, _) => {
                polys.push(ConvexPolygon(
                    real.iter()
                        .map(|i| {
                            (if self.lines().any(|l| l == i.a) {
                                i.a
                            } else {
                                i.b
                            })
                            .points()
                            .find(|p| !self_included.contains(p) && !shared.contains(p))
                            .unwrap()
                        })
                        .chain(real.iter().rev().map(|i| i.point))
                        .collect(),
                ));
            }
            (2, _, 2, 0, _) => {
                let starting_point = self.points().find(|p| !self_included.contains(p)).unwrap();
                polys.push(ConvexPolygon(vec![
                    starting_point,
                    real[0].point,
                    real[1].point,
                ]));
            }
            (1, _, 0, 1, 1) => {
                return vec![self];
            }
            (2, 2, 0, 1, 0) => {
                let contained_point = other.points().find(|p| other_included.contains(p)).unwrap();
                let choice = real[0];
                let dir = (real[0].point - real[1].point).normalize();
                let a = choice
                    .lines()
                    .filter_map(|i| self.lines().find(|l| *l == i))
                    .next()
                    .unwrap();
                let ap = a
                    .points()
                    .find(|p| dir.dot(&(real[0].point - *p).normalize()).signum() == -1.0)
                    .unwrap();
                let b = self.other_line(&a, &ap);
                let bp = b.other_point(&ap);
                let c = self.other_line(&b, &bp);
                let cp = c.other_point(&bp);
                polys.push(ConvexPolygon(vec![
                    contained_point,
                    choice.point,
                    ap,
                    bp,
                    cp,
                    real[1].point,
                ]));
            }
            (1, _, 1, 0, 1) => {
                polys.push(ConvexPolygon(vec![
                    real[0].point,
                    self.points()
                        .filter(|p| !self_included.contains(p) && !shared.contains(p))
                        .next()
                        .unwrap(),
                    *shared.iter().next().unwrap(),
                ]));
            }
            _ => {
                eprintln!("it matched nobody");
                // cursed_subtraction_debug(&self, &other, &i, false);
            }
        };
        cursed_subtraction_debug(&self, &other, &i, false);
        polys
            .iter()
            .flat_map(|poly| poly.triangulate())
            .filter(|tri| tri.area() > 0.0)
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

    fn closest_intersection(&self, intersections: &Vec<&Intersection>, line: Line) -> Intersection {
        intersections
            .iter()
            .filter(|i| i.on_line(line))
            .fold(
                (None, f64::INFINITY.into()),
                |(mut best, mut best_dist): (Option<Intersection>, F64), &intersection| {
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
        ((p.x + 1.0) / 2.0 * 765.0 + 132.5),
        ((-p.y + 1.0) / 2.0 * 765.0),
    )
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
    let mut bb = BoundingBox::new();
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
                let face = Face {
                    eyes: parts[0],
                    noes: parts[1],
                    ears: parts[2],
                    hair: vec![Triangle {
                        a: project(parts[0].vertex),
                        b: project(parts[1].vertex),
                        c: project(parts[2].vertex),
                    }],
                    culled: false,
                };
                bb.expand(&face.hair[0].a);
                bb.expand(&face.hair[0].b);
                bb.expand(&face.hair[0].c);
                faces.push(face);
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

    let mut dbs: Vec<FaceDebug> = vec![];
    let mut drawn: Vec<&mut Face> = vec![];
    // it's time to split hairs
    for (i, face) in faces.iter_mut().enumerate() {
        if face.culled {
            continue;
        }
        let mut cut = false;
        if i == 274 {
            EVIL_MODE.store(true, Ordering::Relaxed);
        } else {
            EVIL_MODE.store(false, Ordering::Relaxed);
        }
        for f2 in drawn.iter() {
            for t2 in f2.hair.iter() {
                let mut haircut: Vec<Triangle> = vec![];
                for t in face.hair.iter() {
                    let mut split = *t - *t2;
                    if (split.len() == 0 || split.len() > 1) && !cut {
                        let mut debug = FaceDebug {
                            face_id: i,
                            bb: BoundingBox::new(),
                            triangle: Triangle {
                                a: project(face.eyes.vertex),
                                b: project(face.noes.vertex),
                                c: project(face.ears.vertex),
                            },
                        };
                        debug.bb.expand(&debug.triangle.a);
                        debug.bb.expand(&debug.triangle.b);
                        debug.bb.expand(&debug.triangle.c);
                        dbs.push(debug);
                        cut = true;
                    }
                    haircut.append(&mut split);
                }
                face.hair = haircut;
            }
        }
        if cut {
            count += 1;
        }
        drawn.push(face);
    }

    // this is where we do rendering :)
    bb.make_square();
    let bb_strings: Vec<_> = dbs
        .into_iter()
        .map(|mut db| {
            db.bb = bb.reproject_bb(&db.bb);
            db.triangle = bb.reproject_triangle(&db.triangle);
            db.to_js_string()
        })
        .collect();
    println!("let bounding_boxes = [");
    for s in bb_strings {
        println!("{}", s);
    }
    println!("];");
    println!("const plot = async () => {{");
    for (i, face) in faces.iter().enumerate() {
        if face.culled {
            continue;
        }
        // if i > 274 {
        //     break;
        // }

        for t in &face.hair {
            // if t.color == Color::Lime {
            //     continue;
            // }
            draw_triangle_js(&bb.reproject_triangle(&t), Color::Lime);
        }
    }
    println!("}}");
    eprintln!("{}", count);
    println!("console.log('{}');", count);
}
