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

#[derive(Copy, Clone, Debug)]
struct FacePart {
    vertex: Point,
    normal: Point,
}

#[derive(Copy, Clone, Debug)]
struct Face {
    eyes: FacePart,
    noes: FacePart,
    ears: FacePart,
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
        .normalize()
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

#[derive(Copy, Clone, Debug)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
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

    fn dot(&self, other: &Point) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
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

fn paper(p: Point) -> (u16, u16) {
    (
        ((p.x + 1.0) / 2.0 * 7650.0 + 1000.0) as u16,
        ((-p.y + 1.0) / 2.0 * 7650.0) as u16,
    )
}

fn canvas(p: Point) -> (f64, f64) {
    (
        ((p.x + 1.0) / 2.0 * 765.0 + 100.0),
        ((-p.y + 1.0) / 2.0 * 765.0),
    )
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
    let contents: String = fs::read_to_string("teapot.obj").unwrap();
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
                            vertex: v[parts[0].parse::<usize>().unwrap() - 1],
                            normal: vn[parts[2].parse::<usize>().unwrap() - 1],
                        }
                    })
                    .collect::<Vec<_>>();
                faces.push(Face {
                    eyes: parts[0],
                    noes: parts[1],
                    ears: parts[2],
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

    let dt = 3.1415 / 2.0;
    let camera = Point {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    let mut count = 0;
    for face in faces {
        let pface = Face {
            eyes: FacePart {
                vertex: translate(rotate(face.eyes.vertex, dt)),
                normal: face.eyes.normal,
            },
            noes: FacePart {
                vertex: translate(rotate(face.noes.vertex, dt)),
                normal: face.noes.normal,
            },
            ears: FacePart {
                vertex: translate(rotate(face.ears.vertex, dt)),
                normal: face.ears.normal,
            },
        };
        let n = pface.calc_normal();
        let c = pface.calc_centroid();
        let which_way = n.dot(&c).signum();
        if which_way < 0.0 {
            continue;
        }
        count += 1;

        let set = vec![
            face.eyes.vertex,
            face.noes.vertex,
            face.ears.vertex,
            face.eyes.vertex,
        ];
        for pp in set.windows(2) {
            let [p, p2] = pp else {
                panic!("haha");
            };
            let p = t2(project(translate(rotate(*p, dt))));
            let p2 = t2(project(translate(rotate(*p2, dt))));
            // draw_line_paper(p, p2, 0, 0);
            draw_line_js(p, p2, 0, 0);
        }
    }
    println!("console.log('{}');", count);
}
