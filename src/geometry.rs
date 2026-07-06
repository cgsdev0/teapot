use ordered_float::OrderedFloat;
use std::array::IntoIter;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::{Add, Mul, Sub};

pub type F64 = OrderedFloat<f64>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FacePart {
    pub vertex: Point,
    pub normal: Point,
}

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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl Hash for Line {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.a <= self.b {
            self.a.hash(state);
            self.b.hash(state);
        } else {
            self.b.hash(state);
            self.a.hash(state);
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

#[derive(Copy, Clone, Debug)]
pub enum BBMode {
    FromCenter,
    FromTopLeft,
}

#[derive(Debug)]
pub struct BoundingBox {
    pub min: Point,
    pub max: Point,
    pub mode: BBMode,
}

impl BoundingBox {
    pub fn new() -> Self {
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
        let xrange = self.reproject1d(point.x.into(), self.min.x.into(), self.max.x.into(), 1030.0);
        let yrange = self.reproject1d(point.y.into(), self.min.y.into(), self.max.y.into(), 765.0);
        Point {
            x: xrange.into(),
            y: yrange.into(),
            z: 0.0.into(),
        }
    }

    pub fn unproject(&self, point: &Point) -> Point {
        let xrange = self.unproject1d(point.x.into(), self.min.x.into(), self.max.x.into(), 1030.0);
        let yrange = self.unproject1d(point.y.into(), self.min.y.into(), self.max.y.into(), 765.0);
        Point {
            x: xrange.into(),
            y: yrange.into(),
            z: 0.0.into(),
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
                // cursed_subtraction_debug(&self, &other, &i, false);
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
                // cursed_subtraction_debug(&self, &other, &i, false);
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
                // cursed_subtraction_debug(&self, &other, &i, false);
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
        // cursed_subtraction_debug(&self, &other, &i, false);
        polys
            .iter()
            .flat_map(|poly| poly.triangulate())
            .filter(|tri| tri.area() > 0.0)
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Face {
    pub eyes: FacePart,
    pub noes: FacePart,
    pub ears: FacePart,
    pub hair: Vec<Triangle>,
    pub culled: bool,
}

impl Hash for Face {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.eyes.hash(state);
        self.noes.hash(state);
        self.ears.hash(state);
    }
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
    pub fn calc_centroid(&self) -> Point {
        let a = self.eyes.vertex;
        let b = self.noes.vertex;
        let c = self.ears.vertex;
        Point {
            x: (a.x + b.x + c.x) / 3.0,
            y: (a.y + b.y + c.y) / 3.0,
            z: (a.z + b.z + c.z) / 3.0,
        }
    }
    pub fn calc_normal(&self) -> Point {
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
pub struct Point {
    pub x: F64,
    pub y: F64,
    pub z: F64,
}

impl std::fmt::Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use the write! macro to format your output into the buffer `f`
        write!(f, "({:.5}, {:.5})", self.x, self.y)
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

    pub fn dot(&self, other: &Point) -> F64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn dist2(&self, other: &Point) -> F64 {
        let x = other.x - self.x;
        let y = other.y - self.y;
        let z = other.z - self.z;
        x * x + y * y + z * z
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

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.x.cmp(&other.x).then(self.y.cmp(&other.y))
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
