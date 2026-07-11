use raylib::prelude::*;

use crate::geometry::Point;

trait Renderer {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType);
    fn with_raylib(&mut self, _f: &mut dyn FnMut(&mut RaylibDrawHandle)) {}
}

pub struct RaylibRenderer<'a> {
    pub d: RaylibDrawHandle<'a>,
}

impl<'a> Renderer for RaylibRenderer<'a> {
    fn draw_line(&mut self, p1: &Point, p2: &Point, color: ColorType) {
        if let Some(color) = color.stroke() {
            self.d.draw_line_v(*p1, *p2, color);
        }
    }
    fn with_raylib(&mut self, f: &mut dyn FnMut(&mut RaylibDrawHandle)) {
        f(&mut self.d);
    }
}

pub struct HpglRenderer {
    current_pen: usize,
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
    Shaded(u8),
}

impl ColorType {
    pub fn pen(&self) -> usize {
        match self {
            ColorType::Rhs => 6,
            ColorType::Cut => 7,
            _ => 0,
        }
    }
    pub fn fill(&self) -> Option<Color> {
        return None;
        match self {
            ColorType::Primary => Some(Color::WHITE.alpha(0.25)),
            ColorType::Lhs => Some(Color::WHITE.alpha(0.25)),
            ColorType::Rhs => Some(Color::RED.alpha(0.25)),
            ColorType::Difference => None,
            ColorType::Selected => Some(Color::LIME.alpha(0.25)),
            ColorType::Cut => Some(Color::from_hex("00AAAA").unwrap().alpha(0.25)),
            ColorType::Dark => None,
            ColorType::Shaded(val) => Some(Color {
                r: *val,
                g: *val,
                b: *val,
                a: 255,
            }),
        }
    }
    pub fn stroke(&self) -> Option<Color> {
        match self {
            ColorType::Lhs => Some(Color::from_hex("666666").unwrap()),
            ColorType::Rhs => Some(Color::RED),
            ColorType::Difference => Some(Color::BLUE),
            ColorType::Selected => Some(Color::LIME),
            ColorType::Cut => Some(Color::from_hex("00AAAA").unwrap()),
            ColorType::Dark => Some(Color::WHITE.alpha(0.1)),
            _ => None,
        }
    }
}
