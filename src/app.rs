use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use web_sys::{console, CanvasRenderingContext2d};
use yew_router::Routable;

use crate::geometry::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FacePart {
    pub vertex: Point,
    pub normal: Option<Point>,
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

#[derive(PartialEq, Eq, Copy, Clone, Debug, Routable)]
pub enum AppView {
    #[at("/")]
    Main,
    #[at("/no-clip")]
    NoClip,
    #[at("/painter/:face")]
    Painter { face: usize },
    #[at("/slice/:face/:idx")]
    SliceView { face: usize, idx: usize },
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub struct AppState {
    pub faces: Vec<Face>,
    pub ctx: Option<CanvasRenderingContext2d>,
    pub bb: BoundingBox,
    pub zoom: BoundingBox,
    pub edges: Vec<Line>,
    pub selected_faces: HashSet<usize>,
    pub view: AppView,
}

#[allow(dead_code)]
fn paper(p: Point) -> (u16, u16) {
    (
        ((p.x.into_inner() + 1.0) / 2.0 * 7650.0 + 1000.0) as u16,
        (((-p.y).into_inner() + 1.0) / 2.0 * 7650.0) as u16,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Color {
    Lime,
    Lhs,
    Rhs,
    Difference,
    Selected,
}

const TEAPOT: &str = include_str!("../teapot.obj");

impl AppState {
    pub fn new() -> Self {
        AppState {
            ctx: None,
            faces: vec![],
            bb: BoundingBox::new(),
            zoom: BoundingBox::new(),
            edges: vec![],
            selected_faces: std::collections::HashSet::new(),
            view: AppView::Main,
        }
    }

    pub fn zoom_to(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let min = self.zoom.unproject(&Point {
            x: x1.min(x2).into(),
            y: y1.min(y2).into(),
            z: 0.0.into(),
        });
        let max = self.zoom.unproject(&Point {
            x: x2.max(x1).into(),
            y: y2.max(y1).into(),
            z: 0.0.into(),
        });
        self.zoom = BoundingBox {
            min,
            max,
            mode: BBMode::FromTopLeft,
        };
    }

    fn draw_triangle(&self, t: &Triangle, color: Color) {
        let ctx = self.ctx.as_ref().unwrap();
        match color {
            Color::Lime => {
                ctx.set_fill_style_str("#ffffff30");
                ctx.set_stroke_style_str("transparent");
            }
            Color::Lhs => {
                ctx.set_stroke_style_str("#666");
            }
            Color::Rhs => {
                ctx.set_fill_style_str("#ff000030");
                ctx.set_stroke_style_str("red");
            }
            Color::Difference => {
                ctx.set_fill_style_str("transparent");
                ctx.set_stroke_style_str("blue");
            }
            Color::Selected => {
                ctx.set_fill_style_str("#00ff0030");
                ctx.set_stroke_style_str("lime");
            }
        }
        ctx.begin_path();
        let (x, y) = self.to_canvas(t.a);
        ctx.move_to(x.into(), y.into());
        let (x, y) = self.to_canvas(t.b);
        ctx.line_to(x.into(), y.into());
        let (x, y) = self.to_canvas(t.c);
        ctx.line_to(x.into(), y.into());
        let (x, y) = self.to_canvas(t.a);
        ctx.line_to(x.into(), y.into());
        ctx.fill();
        ctx.stroke();
    }

    fn draw_line(&self, p1: Point, p2: Point) {
        let ctx = self.ctx.as_ref().unwrap();
        ctx.set_stroke_style_str("red");
        let (x, y) = self.to_canvas(p1);
        ctx.move_to(x.into(), y.into());
        let (x, y) = self.to_canvas(p2);
        ctx.line_to(x.into(), y.into());
        ctx.stroke();
    }

    // TODO: would be nice to refactor this to return Point
    fn to_canvas(&self, p: Point) -> (F64, F64) {
        let new_point = self.zoom.reproject(&Point {
            x: ((p.x + 1.0) / 2.0 * 765.0 + 132.5),
            y: ((-p.y + 1.0) / 2.0 * 765.0),
            z: 0.0.into(),
        });
        (new_point.x, new_point.y)
    }

    fn from_canvas(&self, p: &Point) -> Point {
        let p = self.zoom.unproject(p);
        Point {
            x: ((p.x - 132.5) * 2.0 / 765.0 - 1.0),
            y: (-((p.y * 2.0 / 765.0) - 1.0)),
            z: 0.0.into(),
        }
    }

    fn draw_point(&self, point: Point, color: &str, open: bool) {
        println!("ctx.beginPath();");
        println!("ctx.fillStyle = '{}';", color);
        println!("ctx.strokeStyle = '{}';", color);
        let (x, y) = self.to_canvas(point);
        println!("ctx.arc(zx({}), zy({}), 5, 0, 20 * Math.PI);", x, y);
        if open {
            println!("ctx.stroke();");
        } else {
            println!("ctx.fill();");
        }
    }

    pub fn render(&self) {
        self.clear();
        for (i, face) in self.faces.iter().enumerate() {
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
                let t = self.bb.reproject_triangle(t);
                if self.selected_faces.contains(&i) {
                    self.draw_triangle(&t, Color::Selected);
                } else {
                    self.draw_triangle(&t, Color::Lime);
                }
            }
        }
        for &edge in self.edges.iter() {
            self.draw_line(edge.a, edge.b);
        }
    }

    pub fn clear(&self) {
        let ctx = self.ctx.as_ref().unwrap();
        ctx.set_line_width(1.0);
        ctx.clear_rect(
            0.0,
            0.0,
            ctx.canvas().unwrap().width() as f64,
            ctx.canvas().unwrap().height() as f64,
        );
        ctx.set_fill_style_str("black");
        ctx.fill_rect(
            0.0,
            0.0,
            ctx.canvas().unwrap().width() as f64,
            ctx.canvas().unwrap().height() as f64,
        );
        ctx.set_fill_style_str("#ffffff30");
    }

    pub fn find_edges(&mut self) {
        let mut edges: HashMap<Line, usize> = std::collections::HashMap::new();
        for face in self.faces.iter() {
            if face.culled {
                continue;
            }
            // if i > 274 {
            //     break;
            // }

            for t in &face.hair {
                let t = self.bb.reproject_triangle(t);
                for edge in t.lines() {
                    match edges.get(&edge) {
                        Some(count) => {
                            edges.insert(edge, count + 1);
                        }
                        None => {
                            edges.insert(edge, 1);
                        }
                    }
                }
            }
        }
        self.edges = edges
            .iter()
            .filter(|&(_, &count)| count == 1)
            .map(|(&line, _)| line)
            .collect();
    }

    pub fn reset_zoom(&mut self) {
        self.zoom = BoundingBox::new();
        self.zoom.mode = BBMode::FromTopLeft;
        self.zoom.expand(&Point {
            x: 0.0.into(),
            y: 0.0.into(),
            z: 0.0.into(),
        });
        self.zoom.expand(&Point {
            x: 1030.0.into(),
            y: 765.0.into(),
            z: 0.0.into(),
        });
    }

    pub fn move_pointer(&mut self, x: i32, y: i32) {
        let p = Point {
            x: x.into(),
            y: y.into(),
            z: 0.0.into(),
        };
        let p = self.from_canvas(&p);
        let p = self.bb.unproject(&p);
        let mut dirty = false;
        for (i, face) in self.faces.iter().enumerate() {
            if face.culled {
                continue;
            }
            for t in face.hair.iter() {
                if t.contains(&p) {
                    if !dirty {
                        dirty = true;
                        self.selected_faces.clear();
                    }
                    self.selected_faces.insert(i);
                    break;
                }
            }
        }
        if !dirty && !self.selected_faces.is_empty() {
            dirty = true;
            self.selected_faces.clear();
        }
        if dirty {
            self.render();
        }
    }

    pub fn backface_culling(&mut self) {
        for face in self.faces.iter_mut() {
            let n = face.calc_normal();
            let c = face.calc_centroid().normalize();
            let which_way = n.dot(&c);
            if which_way <= 0.0.into() {
                face.culled = true;
            }
        }
    }

    pub fn reasonable_culling(&mut self) {
        let mut drawn: Vec<&mut Face> = vec![];
        // this is where it gets hairy
        for face in self.faces.iter_mut() {
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
    }
    pub fn partial_culling(&mut self) {
        let mut drawn: Vec<&mut Face> = vec![];
        // it's time to split hairs
        for (i, face) in self.faces.iter_mut().enumerate() {
            if face.culled {
                continue;
            }
            if let AppView::Painter { face } = self.view {
                if i > (face as usize) {
                    break;
                }
            }
            let mut cut = false;
            for f2 in drawn.iter() {
                for t2 in f2.hair.iter() {
                    let mut haircut: Vec<Triangle> = vec![];
                    for t in face.hair.iter() {
                        let mut split = *t - *t2;
                        if (split.is_empty() || split.len() > 1) && !cut {
                            cut = true;
                        }
                        haircut.append(&mut split);
                    }
                    face.hair = haircut;
                }
            }
            drawn.push(face);
        }
        if let AppView::Painter { .. } = self.view {
            // if debug.is_some() {
            self.faces = drawn.into_iter().map(|d| d.clone()).collect();
        }
    }

    pub fn restart(&mut self) {
        console::log_1(&format!("view: {:?}", self.view).into());
        self.clear();
        self.edges.clear();
        self.faces.clear();
        let mut v: Vec<Point> = vec![];
        let mut vn: Vec<Point> = vec![];
        let dt: F64 = (std::f64::consts::PI / 2.0).into();
        for line in TEAPOT.lines() {
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
                                normal: match parts.len() {
                                    1 | 2 => None,
                                    _ => Some(vn[parts[2].parse::<usize>().unwrap() - 1]),
                                },
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
                    self.bb.expand(&face.hair[0].a);
                    self.bb.expand(&face.hair[0].b);
                    self.bb.expand(&face.hair[0].c);
                    self.faces.push(face);
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

        let _count = 0;
        self.faces.sort();

        self.backface_culling();
        self.reasonable_culling();
        match self.view {
            AppView::NoClip => {}
            _ => {
                self.partial_culling();
            }
        };

        self.bb.make_square();
        self.find_edges();
    }

    pub fn start(&mut self, ctx: CanvasRenderingContext2d) {
        self.ctx = Some(ctx);
        self.reset_zoom();
    }
}
