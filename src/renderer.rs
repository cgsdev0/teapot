use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

use crate::bounding_box::BoundingBox;
use raylib::prelude::*;
use uuid::Uuid;

use crate::geometry::{Point, Triangle};

pub trait Renderer {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType);
    fn with_raylib(&mut self, _f: &mut dyn FnMut(&mut RaylibDrawHandle)) {}
    fn draw_triangle(&mut self, t: &Triangle, color: ColorType);
}

pub struct RaylibRenderer<'a> {
    pub d: RaylibDrawHandle<'a>,
    pub zoom: BoundingBox,
}

impl<'a> Renderer for RaylibRenderer<'a> {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType) {
        if let Some(color) = color.stroke() {
            let p1 = to_canvas(p1);
            let p2 = to_canvas(p2);
            let p1 = self.zoom.reproject(&p1);
            let p2 = self.zoom.reproject(&p2);
            self.d.draw_line_v(p1, p2, color);
        }
    }
    fn with_raylib(&mut self, f: &mut dyn FnMut(&mut RaylibDrawHandle)) {
        f(&mut self.d);
    }
    fn draw_triangle(&mut self, t: &Triangle, color: ColorType) {
        let a = to_canvas(&t.a);
        let b = to_canvas(&t.b);
        let c = to_canvas(&t.c);
        let a = self.zoom.reproject(&a);
        let b = self.zoom.reproject(&b);
        let c = self.zoom.reproject(&c);
        let ab = a - b;
        let ac = a - c;
        let cross = ab.x * ac.y - ab.y * ac.x;
        // we need to sort to clockwise
        if let Some(fill) = color.fill() {
            match cross.signum() {
                -1.0 => self.d.draw_triangle(a, b, c, fill),
                _ => self.d.draw_triangle(a, c, b, fill),
            };
        }
        if let Some(stroke) = color.stroke() {
            match cross.signum() {
                -1.0 => self.d.draw_triangle_lines(a, b, c, stroke),
                _ => self.d.draw_triangle_lines(a, c, b, stroke),
            };
        }
    }
}

#[derive(Default)]
pub struct Optimizer {
    paths: Vec<Vec<(i32, i32)>>,
}

impl Optimizer {
    /// returns whether or not it simplified anything
    fn simplify(&mut self) -> bool {
        let mut updated = false;
        let mut new_paths: Vec<Vec<(i32, i32)>> = vec![];
        let mut starts: HashMap<(i32, i32), usize> = HashMap::new();
        let mut ends: HashMap<(i32, i32), usize> = HashMap::new();
        let mut inverts: HashMap<(i32, i32), (i32, i32)> = HashMap::new();
        for (i, path) in self.paths.iter().enumerate() {
            let first = path[0];
            let last = path[path.len() - 1];
            if let Some(j) = starts.get(&first) {
                updated = true;
                new_paths.push(
                    path.iter()
                        .rev()
                        .chain(self.paths[*j].iter().skip(1))
                        .copied()
                        .collect(),
                );
                starts.remove(&first);
                ends.remove(inverts.get(&first).unwrap());
            } else if let Some(j) = starts.get(&last) {
                updated = true;
                new_paths.push(
                    path.iter()
                        .chain(self.paths[*j].iter().skip(1))
                        .copied()
                        .collect(),
                );
                starts.remove(&last);
                ends.remove(inverts.get(&last).unwrap());
            } else if let Some(j) = ends.get(&first) {
                updated = true;
                new_paths.push(
                    self.paths[*j]
                        .iter()
                        .chain(path.iter().skip(1))
                        .copied()
                        .collect(),
                );
                starts.remove(&first);
                ends.remove(inverts.get(&first).unwrap());
            } else if let Some(j) = ends.get(&last) {
                updated = true;
                new_paths.push(
                    self.paths[*j]
                        .iter()
                        .chain(path.iter().rev().skip(1))
                        .copied()
                        .collect(),
                );
                starts.remove(&last);
                ends.remove(inverts.get(&last).unwrap());
            } else {
                starts.insert(first, i);
                ends.insert(last, i);
                inverts.insert(first, last);
                inverts.insert(last, first);
            }
        }
        if updated {
            for (_, i) in starts {
                new_paths.push(self.paths[i].clone());
            }
            self.paths = new_paths;
        }
        updated
    }
}

pub struct HpglRenderer {
    current_pen: usize,
    writer: BufWriter<File>,
    pens: HashMap<usize, Optimizer>,
}

impl Default for HpglRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HpglRenderer {
    pub fn new() -> Self {
        let uuid = Uuid::now_v7();
        let fname = "hpgl/".to_owned() + uuid.to_string().as_str() + ".hpgl";
        eprintln!("Creating file: {}", fname);
        let file = File::create(fname).unwrap();
        HpglRenderer {
            current_pen: 0,
            writer: BufWriter::new(file),
            pens: HashMap::new(),
        }
    }
    pub fn optimize(&mut self) {
        for (_, optimizer) in self.pens.iter_mut() {
            loop {
                let out = optimizer.simplify();
                if !out {
                    break;
                }
            }
        }
    }
    pub fn write(&mut self) {
        writeln!(self.writer, "IN;").unwrap();
        for (pen, optimizer) in self.pens.iter_mut() {
            writeln!(self.writer, "SP{};", pen).unwrap();
            let mut path = optimizer.paths[0].clone();
            optimizer.paths.remove(0);
            loop {
                let (x, y) = &path[0];
                writeln!(self.writer, "PU {},{};", x, y).unwrap();
                let (x, y) = &path[1];
                writeln!(self.writer, "PD {},{};", x, y).unwrap();
                for point in path.iter().skip(2) {
                    let (x, y) = &point;
                    writeln!(self.writer, "PA {},{};", x, y).unwrap();
                }
                if optimizer.paths.is_empty() {
                    break;
                }
                let mut min = f64::INFINITY;
                let end = path[path.len() - 1];
                let mut closest: Option<Closest> = None;
                for (i, p) in optimizer.paths.iter().enumerate() {
                    let d1 = dist(&p[0], &end);
                    if d1 < min {
                        min = d1;
                        closest = Some(Closest::Start(i));
                    }
                    let d2 = dist(&p[p.len() - 1], &end);
                    if d2 < min {
                        min = d2;
                        closest = Some(Closest::End(i));
                    }
                }
                match closest {
                    Some(Closest::Start(i)) => {
                        path = optimizer.paths.remove(i);
                    }
                    Some(Closest::End(i)) => {
                        path = optimizer.paths.remove(i);
                    }
                    None => {
                        break;
                    }
                }
            }
        }
    }
}

fn dist(a: &(i32, i32), b: &(i32, i32)) -> f64 {
    let (x1, y1) = a;
    let (x2, y2) = b;
    let dx = (x2 - x1) as f64;
    let dy = (y2 - y1) as f64;
    dx * dx + dy * dy
}

enum Closest {
    Start(usize),
    End(usize),
}

fn to_paper(p: &Point) -> (i32, i32) {
    let new_point = Point {
        x: ((p.x + 1.0) / 2.0 * 7650.0 + 1325.0),
        y: ((-p.y + 1.0) / 2.0 * 7650.0),
        z: 0.0,
    };
    (new_point.x as i32, new_point.y as i32)
}

fn to_canvas(p: &Point) -> Point {
    Point {
        x: ((-p.x + 1.0) / 2.0 * 765.0 + 132.5),
        y: ((-p.y + 1.0) / 2.0 * 765.0),
        z: 0.0,
    }
}

impl Renderer for HpglRenderer {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType) {
        let pen = color.pen();
        if pen > 0 {
            let optimizer = self.pens.entry(pen).or_default();
            let (x, y) = to_paper(p1);
            let (x2, y2) = to_paper(p2);
            optimizer.paths.push(vec![(x, y), (x2, y2)]);
        }
    }
    fn draw_triangle(&mut self, t: &Triangle, color: ColorType) {
        for line in t.lines() {
            self.draw_line(&line.a, &line.b, color);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorType {
    Primary,
    Contour,
    Outline,
    Lhs,
    Rhs,
    Difference,
    Selected,
    Cut,
    Dark,
    Pink,
    Blue,
    Black,
    Shaded(u8),
}

impl ColorType {
    pub fn pen(&self) -> usize {
        match self {
            // TODO
            ColorType::Outline => 6,
            ColorType::Contour => 7,
            _ => 0,
        }
    }
    pub fn fill(&self) -> Option<Color> {
        match self {
            ColorType::Primary => Some(Color::RED.alpha(0.25)),
            ColorType::Lhs => Some(Color::WHITE.alpha(0.25)),
            ColorType::Rhs => Some(Color::RED.alpha(0.25)),
            ColorType::Selected => Some(Color::LIME.alpha(0.25)),
            ColorType::Cut => Some(Color::from_hex("00AAAA").unwrap().alpha(0.25)),
            ColorType::Shaded(val) => Some(Color {
                r: *val,
                g: *val,
                b: *val,
                a: 255,
            }),
            _ => None,
        }
    }
    pub fn stroke(&self) -> Option<Color> {
        match self {
            ColorType::Contour => Some(Color::BLACK),
            ColorType::Outline => Some(Color::BLACK),
            ColorType::Primary => Some(Color::BLACK),
            ColorType::Lhs => Some(Color::from_hex("666666").unwrap()),
            ColorType::Rhs => Some(Color::RED),
            ColorType::Difference => Some(Color::BLUE),
            ColorType::Selected => Some(Color::LIME),
            ColorType::Cut => Some(Color::from_hex("00AAAA").unwrap()),
            ColorType::Dark => Some(Color::WHITE.alpha(0.1)),
            ColorType::Pink => Some(Color::from_hex("ff3388").unwrap()),
            ColorType::Blue => Some(Color::from_hex("0099ff").unwrap()),
            ColorType::Black => Some(Color::from_hex("333333").unwrap()),
            _ => None,
        }
    }
}
