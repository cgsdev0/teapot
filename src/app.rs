use crate::geometry::BoundingBox;
use raylib::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use raylib::prelude::RaylibDrawHandle;

use crate::geometry::*;
use crate::navigator::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FacePart {
    pub vertex: Point,
    pub normal: Option<Point>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Face {
    pub id: usize,
    pub eyes: FacePart,
    pub noes: FacePart,
    pub ears: FacePart,
    pub hair: Triangle,
    pub haircut: Vec<Triangle>,
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

pub struct DebugView {
    pub tri: Triangle,
    pub haircut: Vec<Triangle>,
    pub cutter: Triangle,
}

pub struct AppState {
    pub faces: Vec<Face>,
    pub bb: BoundingBox,
    pub edges: Vec<Line>,
    pub selected_faces: HashSet<usize>,
    pub nav: Navigator,
    pub debug_view: Option<DebugView>,
    pub selection: Option<(Vector2, Vector2)>,
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
enum ColorType {
    Primary,
    Lhs,
    Rhs,
    Difference,
    Selected,
    Cut,
    Dark,
}

impl ColorType {
    pub fn fill(&self) -> Option<Color> {
        match self {
            ColorType::Primary => Some(Color::WHITE.alpha(0.25)),
            ColorType::Lhs => Some(Color::WHITE.alpha(0.25)),
            ColorType::Rhs => Some(Color::RED.alpha(0.25)),
            ColorType::Difference => None,
            ColorType::Selected => Some(Color::LIME.alpha(0.25)),
            ColorType::Cut => Some(Color::from_hex("00AAAA").unwrap().alpha(0.25)),
            ColorType::Dark => None,
        }
    }
    pub fn stroke(&self) -> Option<Color> {
        match self {
            ColorType::Primary => None,
            ColorType::Lhs => Some(Color::from_hex("666666").unwrap()),
            ColorType::Rhs => Some(Color::RED),
            ColorType::Difference => Some(Color::BLUE),
            ColorType::Selected => Some(Color::LIME),
            ColorType::Cut => Some(Color::from_hex("00AAAA").unwrap()),
            ColorType::Dark => Some(Color::WHITE.alpha(0.1)),
        }
    }
}

const TEAPOT: &str = include_str!("../utah-beetle-only.obj");

impl Default for AppState {
    fn default() -> Self {
        AppState::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            faces: vec![],
            bb: BoundingBox::new(),
            edges: vec![],
            nav: Navigator::new(),
            selected_faces: std::collections::HashSet::new(),
            debug_view: None,
            selection: None,
        }
    }
    pub fn update(&mut self, rl: &mut RaylibHandle) {
        // arrow keys
        match self.nav.current() {
            AppView::SliceView { face, idx } => {
                if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
                    if idx > 0 {
                        self.nav.push(AppView::SliceView { face, idx: idx - 1 });
                        self.restart();
                    }
                } else if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
                    self.nav.push(AppView::SliceView { face, idx: idx + 1 });
                    self.restart();
                }
            }
            _ => {}
        };
        // mouse stuff
        let pos = rl.get_mouse_position();
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            self.selection = Some((pos, pos));
        }
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
            self.nav.reset_zoom();
        }
        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(selection) = self.selection {
                let delta = selection.1 - selection.0;
                let dx = delta.x.abs();
                let dy = delta.y.abs();
                if dx < 5.0 && dy < 5.0 {
                    self.selection = None;
                    self.pointer_click(pos.x, pos.y);
                    return;
                }
                // apply selection
                self.nav.zoom_to(
                    selection.0.x.into(),
                    selection.0.y.into(),
                    selection.1.x.into(),
                    selection.1.y.into(),
                );
                self.selection = None;
            }
        }
        if let Some(selection) = &mut self.selection {
            selection.1 = rl.get_mouse_position();
        } else {
            self.pointer_move(pos.x, pos.y);
        }
    }

    fn draw_triangle(&self, d: &mut RaylibDrawHandle, t: &Triangle, color: ColorType) {
        let t = self.bb.reproject_triangle(t);
        let a = self.to_canvas(t.a);
        let b = self.to_canvas(t.b);
        let c = self.to_canvas(t.c);
        let ab = a - b;
        let ac = a - c;
        let cross = ab.x * ac.y - ab.y * ac.x;
        // we need to sort to clockwise
        if let Some(fill) = color.fill() {
            match cross.signum() {
                -1.0 => d.draw_triangle(a, b, c, fill),
                _ => d.draw_triangle(a, c, b, fill),
            };
        }
        if let Some(stroke) = color.stroke() {
            match cross.signum() {
                -1.0 => d.draw_triangle_lines(a, b, c, stroke),
                _ => d.draw_triangle_lines(a, c, b, stroke),
            };
        }
    }

    // fn draw_line(&self, p1: Point, p2: Point) {
    //     let ctx = self.ctx.as_ref().unwrap();
    //     ctx.set_stroke_style_str("red");
    //     let (x, y) = self.to_canvas(p1);
    //     ctx.move_to(x.into(), y.into());
    //     let (x, y) = self.to_canvas(p2);
    //     ctx.line_to(x.into(), y.into());
    //     ctx.stroke();
    // }

    fn to_canvas(&self, p: Point) -> Vector2 {
        let new_point = self.nav.zoom.reproject(&Point {
            x: ((p.x + 1.0) / 2.0 * 765.0 + 132.5),
            y: ((-p.y + 1.0) / 2.0 * 765.0),
            z: 0.0.into(),
        });
        Vector2 {
            x: new_point.x.into_inner() as f32,
            y: new_point.y.into_inner() as f32,
        }
    }

    fn from_canvas(&self, p: &Point) -> Point {
        let p = self.nav.zoom.unproject(p);
        Point {
            x: ((p.x - 132.5) * 2.0 / 765.0 - 1.0),
            y: (-((p.y * 2.0 / 765.0) - 1.0)),
            z: 0.0.into(),
        }
    }

    // fn draw_point(&self, point: Point, color: &str, open: bool) {
    //     println!("ctx.beginPath();");
    //     println!("ctx.fillStyle = '{}';", color);
    //     println!("ctx.strokeStyle = '{}';", color);
    //     let (x, y) = self.to_canvas(point);
    //     println!("ctx.arc(zx({}), zy({}), 5, 0, 20 * Math.PI);", x, y);
    //     if open {
    //         println!("ctx.stroke();");
    //     } else {
    //         println!("ctx.fill();");
    //     }
    // }

    pub fn render(&self, d: &mut RaylibDrawHandle) {
        self.clear(d);
        let view = self.nav.current();
        match view {
            AppView::SliceView { .. } => self.render_debug(d),
            _ => self.render_standard(d),
        };
        if let Some(selection) = self.selection {
            let pos = selection.0;
            let size = selection.1 - selection.0;
            d.draw_rectangle_lines(
                pos.x as i32,
                pos.y as i32,
                size.x as i32,
                size.y as i32,
                Color::RED,
            );
        }
    }

    pub fn render_debug(&self, d: &mut RaylibDrawHandle) {
        let Some(debug_view) = &self.debug_view else {
            self.render_standard(d);
            return;
        };
        for face in self.faces.iter() {
            if face.culled {
                continue;
            }
            for t in &face.haircut {
                self.draw_triangle(d, t, ColorType::Dark);
            }
        }
        let DebugView {
            tri,
            haircut,
            cutter,
        } = debug_view;
        self.draw_triangle(d, tri, ColorType::Lhs);
        self.draw_triangle(d, cutter, ColorType::Rhs);
        for cut in haircut {
            self.draw_triangle(d, cut, ColorType::Difference);
        }
    }

    pub fn render_standard(&self, d: &mut RaylibDrawHandle) {
        for face in self.faces.iter() {
            if face.culled {
                continue;
            }
            // if i > 274 {
            //     break;
            // }

            for t in &face.haircut {
                // if t.color == ColorType::Primary {
                //     continue;
                // }
                if self.selected_faces.contains(&face.id) {
                    self.draw_triangle(d, t, ColorType::Selected);
                } else {
                    self.draw_triangle(d, t, ColorType::Primary);

                    // self.draw_triangle(
                    //     &t,
                    //     match face.haircut.len() {
                    //         1 => ColorType::Primary,
                    //         _ => ColorType::Cut,
                    //     },
                    // );
                }
            }
        }
        // for &edge in self.edges.iter() {
        //     self.draw_line(edge.a, edge.b);
        // }
    }

    pub fn clear(&self, d: &mut RaylibDrawHandle) {
        d.clear_background(Color::BLACK);
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

            for t in &face.haircut {
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

    pub fn pointer_click(&mut self, x: f32, y: f32) {
        println!("Clicked: {:?}", self.selected_faces);
        let faces: Vec<_> = self.selected_faces.iter().take(2).collect();
        match self.nav.current() {
            AppView::Painter { .. } | AppView::SliceView { .. } => match faces.len() {
                1 => {
                    self.nav.push(AppView::SliceView {
                        face: SliceThing::OneFace(*faces[0]),
                        idx: 1,
                    });
                    self.restart();
                }
                2 => {
                    self.nav.push(AppView::SliceView {
                        face: SliceThing::TwoFace(*faces[1], *faces[0]),
                        idx: 1,
                    });
                    self.restart();
                }
                _ => (),
            },
            AppView::Main if !faces.is_empty() => {
                self.nav.push(AppView::Painter {
                    face: *faces.into_iter().last().unwrap(),
                });
                self.restart();
            }
            _ => (),
        };
    }

    pub fn pointer_move(&mut self, x: f32, y: f32) {
        let p = Point {
            x: (x as f64).into(),
            y: (y as f64).into(),
            z: 0.0.into(),
        };
        let p = self.from_canvas(&p);
        let p = self.bb.unproject(&p);
        let mut dirty = false;
        for (i, face) in self.faces.iter().enumerate() {
            if face.culled {
                continue;
            }
            for t in face.haircut.iter() {
                if t.contains(&p) {
                    if !dirty {
                        dirty = true;
                        self.selected_faces.clear();
                    }
                    self.selected_faces.insert(face.id);
                    break;
                }
            }
        }
        if !dirty && !self.selected_faces.is_empty() {
            self.selected_faces.clear();
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
            let mut a_occluded = false;
            let mut b_occluded = false;
            let mut c_occluded = false;
            for f2 in drawn.iter() {
                if f2.hair.contains(&face.hair.a) {
                    a_occluded = true;
                }
                if f2.hair.contains(&face.hair.b) {
                    b_occluded = true;
                }
                if f2.hair.contains(&face.hair.c) {
                    c_occluded = true;
                }
            }
            if a_occluded && b_occluded && c_occluded {
                face.culled = true;
            }
            drawn.push(face);
        }
    }
    pub fn partial_culling(&mut self) {
        let view = self.nav.current();
        let mut drawn: Vec<&mut Face> = vec![];
        let (view_face, cutter_face, view_cut_idx) = match view {
            AppView::SliceView { face, idx } => match face {
                SliceThing::OneFace(fid) => (fid, fid, idx),
                SliceThing::TwoFace(f1, f2) => (f1, f2, idx),
            },
            _ => (0, 0, 0),
        };
        let mut cut_idx = 0;
        // it's time to split hairs
        'cut: for (i, face) in self.faces.iter_mut().enumerate() {
            if face.culled {
                continue;
            }
            if let AppView::Painter { face } = view {
                // TODO: check face IDs?
                if i > face {
                    // console::log_1(&format!("breaking cuz {} > {}", i, face).into());
                    break;
                }
            }
            for f2 in drawn.iter() {
                let mut haircut: Vec<Triangle> = vec![];
                for t in face.haircut.iter() {
                    let mut split = *t - f2.hair;
                    if split.len() > 1
                        || (split.len() == 1 && split[0] != *t)
                        || (cutter_face != view_face
                            && f2.id == cutter_face
                            && face.id == view_face)
                    {
                        // console::log_1(&format!("face id: {}, view: {}, cut: {}, view_cut: {}", face.id, view_face, cut_idx, view_cut_idx).into());
                        if face.id == view_face || f2.id == cutter_face {
                            cut_idx += 1;
                            if cut_idx == view_cut_idx {
                                self.debug_view = Some(DebugView {
                                    tri: *t,
                                    haircut: split,
                                    cutter: f2.hair,
                                });
                                return;
                            }
                        }
                    }
                    haircut.append(&mut split);
                }
                if haircut.is_empty() {
                    face.culled = true;
                    continue 'cut;
                }
                face.haircut = haircut;
            }
            drawn.push(face);
        }
        if let AppView::Painter { .. } = view {
            // if debug.is_some() {
            self.faces = drawn.into_iter().map(|d| d.clone()).collect();
        }
    }

    pub fn restart(&mut self) {
        // console::log_1(&format!("view: {:?}", self.view).into());
        self.edges.clear();
        self.faces.clear();
        self.debug_view = None;
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
                    let tri = Triangle {
                        a: project(parts[0].vertex),
                        b: project(parts[1].vertex),
                        c: project(parts[2].vertex),
                    };
                    let face = Face {
                        id: 0,
                        eyes: parts[0],
                        noes: parts[1],
                        ears: parts[2],
                        hair: tri,
                        haircut: vec![tri],
                        culled: false,
                    };
                    self.bb.expand(&face.hair.a);
                    self.bb.expand(&face.hair.b);
                    self.bb.expand(&face.hair.c);
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
        for (z, face) in self.faces.iter_mut().enumerate() {
            face.id = z;
        }

        self.backface_culling();
        // self.reasonable_culling();
        self.partial_culling();

        self.bb.make_square();
        self.find_edges();
    }
}
