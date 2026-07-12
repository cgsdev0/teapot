use raylib::prelude::*;

use crate::geometry::{Point, Triangle};

pub trait Renderer {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType);
    fn with_raylib(&mut self, _f: &mut dyn FnMut(&mut RaylibDrawHandle)) {}
    fn draw_triangle(&mut self, t: &Triangle, color: ColorType);
}

pub struct RaylibRenderer<'a> {
    pub d: RaylibDrawHandle<'a>,
}

impl<'a> Renderer for RaylibRenderer<'a> {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType) {
        if let Some(color) = color.stroke() {
            self.d.draw_line_v(to_canvas(p1), to_canvas(p2), color);
        }
    }
    fn with_raylib(&mut self, f: &mut dyn FnMut(&mut RaylibDrawHandle)) {
        f(&mut self.d);
    }
    fn draw_triangle(&mut self, t: &Triangle, color: ColorType) {
        let a = to_canvas(&t.a);
        let b = to_canvas(&t.b);
        let c = to_canvas(&t.c);
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

pub struct HpglRenderer {
    current_pen: usize,
}

impl Default for HpglRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HpglRenderer {
    pub fn new() -> Self {
        HpglRenderer { current_pen: 0 }
    }
}

fn to_paper(p: &Point) -> (i32, i32) {
    let new_point = Point {
        x: ((p.x + 1.0) / 2.0 * 7650.0 + 1325.0),
        y: ((-p.y + 1.0) / 2.0 * 7650.0),
        z: 0.0,
    };
    (new_point.x as i32, new_point.y as i32)
}

fn to_canvas(p: &Point) -> Vector2 {
    let new_point = &Point {
        x: ((-p.x + 1.0) / 2.0 * 765.0 + 132.5),
        y: ((-p.y + 1.0) / 2.0 * 765.0),
        z: 0.0,
    };
    Vector2 {
        x: new_point.x as f32,
        y: new_point.y as f32,
    }
}

impl Renderer for HpglRenderer {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType) {
        let pen = color.pen();
        if pen > 0 {
            if self.current_pen != pen {
                println!("SP{};", pen);
                self.current_pen = pen;
            }
            let (x, y) = to_paper(p1);
            println!("PU {},{};", x, y);
            let (x, y) = to_paper(p2);
            println!("PD {},{};", x, y);
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
            ColorType::Black => 5,
            ColorType::Rhs | ColorType::Pink => 6,
            ColorType::Cut | ColorType::Blue => 7,
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
