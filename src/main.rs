// based on the video https://www.youtube.com/watch?v=qjWkNZ0SXfo
// by tsoding :D

use std::fmt::Display;
use std::fs;
use std::ops::{Add, Mul, Sub};
fn set((x, y): (u16, u16), frame: &mut [u8]) {
    if y >= 38 {
        return;
    }
    if x >= 96 {
        return;
    }
    let byte = (y * 96 + x) / 8;
    let bit = 7 - x % 8;
    frame[byte as usize] |= 1 << bit;
}

fn draw_line_js(p1: Point, p2: Point, ox: u16, oy: u16) {
    let (x, y) = canvas(p1);
    println!("ctx.moveTo({}, {});", x, y);
    let (x, y) = canvas(p2);
    println!("ctx.lineTo({}, {});ctx.stroke();", x, y);
}

fn draw_line_paper(p1: Point, p2: Point, ox: u16, oy: u16) {
    let (x, y) = paper(p1);
    println!("PU{},{};", x + ox, y + oy);
    let (x, y) = paper(p2);
    println!("PD{},{};", x + ox, y + oy);
}

fn draw_line(p1: Point, p2: Point, frame: &mut [u8]) {
    let (x, y) = paper(p1);
    println!("PU{},{}", x, y);
    let (x, y) = paper(p2);
    println!("PD{},{}", x, y);
}

#[derive(Copy, Clone)]
struct FacePart {
    vertex: u64,
    normal: u64,
}

#[derive(Copy, Clone)]
struct Face {
    x: FacePart,
    y: FacePart,
    z: FacePart,
}

#[derive(Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
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

fn paper(p: Point) -> (u16, u16) {
    (
        ((p.x + 1.0) / 2.0 * 7650.0 + 1000.0) as u16,
        ((-p.y + 1.0) / 2.0 * 7650.0) as u16,
    )
}

fn canvas(p: Point) -> (f64, f64) {
    (((p.x + 1.0) / 2.0 * 1000.0), ((-p.y + 1.0) / 2.0 * 1000.0))
}

fn screen(p: Point) -> (u16, u16) {
    (
        ((p.x + 1.0) / 2.0 * 96.0) as u16,
        ((-p.y + 1.0) / 2.0 * 38.0) as u16,
    )
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
        z: p.z,
    }
}

fn translate(p: Point) -> Point {
    Point {
        x: p.x,
        y: p.y - 0.9,
        z: p.z + 5.0,
    }
}

fn rotate(p: Point, angle: f64) -> Point {
    let c = angle.cos();
    let s = angle.sin();
    Point {
        x: p.x * c - p.z * s,
        z: p.x * s + p.z * c,
        y: p.y,
    }
}

fn main() {
    let contents: String = fs::read_to_string("teapot_final5.obj").unwrap();
    let mut faces: Vec<Face> = vec![];
    let mut v: Vec<Point> = vec![];
    let mut vn: Vec<Point> = vec![];
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
                            vertex: parts[0].parse::<u64>().unwrap(),
                            normal: parts[2].parse::<u64>().unwrap(),
                        }
                    })
                    .collect::<Vec<_>>();
                faces.push(Face {
                    x: parts[0],
                    y: parts[1],
                    z: parts[2],
                });
            }
            "v" => {
                v.push(Point {
                    x: parts[1].parse::<f64>().unwrap(),
                    y: parts[2].parse::<f64>().unwrap(),
                    z: parts[3].parse::<f64>().unwrap(),
                });
            }
            "vn" => {
                vn.push(Point {
                    x: parts[1].parse::<f64>().unwrap(),
                    y: parts[2].parse::<f64>().unwrap(),
                    z: parts[3].parse::<f64>().unwrap(),
                });
            }
            _ => {}
        }
    }

    // let frame = 6;

    //     let dt = 3.1415 / 2.0;
    //     for set in edges {
    //         for pp in set.windows(2) {
    //             let [p, p2] = pp else { panic!("haha"); };
    //             let p = t2(project(translate(rotate(points[p-1], dt))));
    //             let p2 = t2(project(translate(rotate(points[p2-1], dt))));
    //             draw_line_paper(p, p2, 0, 0);
    //             // draw_line_js(p, p2, 0, 0);
    //         }
    // }
}
