use crate::geometry::BoundingBox;
use crate::renderer::ColorType;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::string::clip::ClipRule;
use i_triangle::float::triangulatable::Triangulatable;
use itertools::Itertools;
use raylib::prelude::*;
use std::collections::HashSet;

use raylib::prelude::RaylibDrawHandle;

use crate::geometry::*;
use crate::navigator::*;
use crate::renderer::*;

const TEAPOT: &str = include_str!("../models/teapot.obj");

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

pub struct Plane {
    pub point: Point,
    pub offset: f64,
}

impl Point {
    pub fn dist_from_plane(&self, plane: &Plane) -> f64 {
        self.dot(&plane.point) + plane.offset
    }
}

impl Line {
    pub fn plane_intersection(&self, plane: &Plane) -> Vec<Point> {
        let mut result = vec![];
        let ad = self.a.dist_from_plane(plane);
        let bd = self.b.dist_from_plane(plane);

        let a_on_plane = ad.abs() <= f64::EPSILON;
        let b_on_plane = bd.abs() <= f64::EPSILON;

        if a_on_plane {
            result.push(self.a);
        }
        if b_on_plane {
            result.push(self.b);
        }
        if a_on_plane && b_on_plane {
            return result;
        }
        if ad * bd >= f64::EPSILON {
            return result;
        }
        let t = ad / (ad - bd);
        result.push(self.a + t * (self.b - self.a));
        result
    }
}

impl Triangle {
    pub fn plane_intersection(&self, plane: &Plane) -> Vec<Point> {
        let mut result = vec![];
        for line in self.lines() {
            let mut i = line.plane_intersection(plane);
            result.append(&mut i);
        }
        result.sort_unstable();
        result.dedup();
        result
    }
}

impl Face {
    pub fn as_triangle(&self) -> Triangle {
        Triangle {
            a: self.eyes.vertex,
            b: self.noes.vertex,
            c: self.ears.vertex,
        }
    }
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
    // TODO:
    // - slice model with planes perpendicular to light sources
    // - save resulting lines by octave
    // - for each face, draw some, all, or no lines depending on dot product
    pub fn hatch(&self, light: &Point) -> Vec<Line> {
        let normal = self.calc_normal();
        let dot = light.dot(&normal);
        // let dot = dot * dot * dot;
        let density = (1.0 - dot) * 1600.0;
        // let color = ColorType::Shaded(l);
        let proj_normal = project(normal - (*light * 1.9)).normalize();
        let mut bb = BoundingBox::new();
        for p in self.hair.points() {
            bb.expand(&p);
        }
        let mut lines: Vec<Line> = vec![];
        let bb_diagonal = bb.min.dist(&bb.max);
        let n_lines = (bb_diagonal * density) as usize;
        for l in 0..n_lines {
            lines.push(Line {
                a: Point {
                    x: l as f64 / -density,
                    y: -100.0,
                    z: 0.0,
                },
                b: Point {
                    x: l as f64 / -density,
                    y: 100.0,
                    z: 0.0,
                },
            });
            lines.push(Line {
                a: Point {
                    x: l as f64 / density,
                    y: -100.0,
                    z: 0.0,
                },
                b: Point {
                    x: l as f64 / density,
                    y: 100.0,
                    z: 0.0,
                },
            });
        }
        lines
            .into_iter()
            .filter_map(|l| {
                let points: Vec<_> = [l.a, l.b]
                    .into_iter()
                    .map(|p| Point {
                        x: p.y * proj_normal.y - p.x * proj_normal.x + bb.min.x,
                        y: -p.y * proj_normal.x - p.x * proj_normal.y + bb.min.y,
                        z: 0.0,
                    })
                    .collect();
                let haircut: Vec<_> = self.haircut.iter().map(|t| vec![t.a, t.b, t.c]).collect();
                let result = points.clip_by(
                    &haircut,
                    FillRule::EvenOdd,
                    ClipRule {
                        invert: false,
                        boundary_included: false,
                    },
                );
                match result.as_slice() {
                    [pair] => Some(Line {
                        a: pair[0],
                        b: pair[1],
                    }),
                    _ => None,
                }
            })
            .collect()
    }
}

pub struct DebugView {
    pub tri: Triangle,
    pub haircut: Vec<Triangle>,
    pub cutter: Triangle,
}

#[derive(Debug)]
pub struct Edge {
    pub line: Line,
    pub face_ids: Vec<usize>,
    pub cut: Vec<Line>,
}

pub struct AppState {
    pub faces: Vec<Face>,
    pub bb: BoundingBox,
    pub edges: Vec<Edge>,
    pub contours: Vec<Line>,
    pub selected_faces: HashSet<usize>,
    pub nav: Navigator,
    pub debug_view: Option<DebugView>,
    pub selection: Option<(Vector2, Vector2)>,
}

#[allow(dead_code)]
fn paper(p: Point) -> (u16, u16) {
    (
        ((p.x + 1.0) / 2.0 * 7650.0 + 1000.0) as u16,
        ((-p.y + 1.0) / 2.0 * 7650.0) as u16,
    )
}

fn project(p: Point) -> Point {
    Point {
        x: p.x / (p.z / 4.0),
        y: p.y / (p.z / 4.0),
        z: 0.0,
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
            contours: vec![],
            nav: Navigator::new(),
            selected_faces: HashSet::new(),
            debug_view: None,
            selection: None,
        }
    }
    pub fn update(&mut self, rl: &mut RaylibHandle) {
        // arrow keys
        // match self.nav.current() {
        //     AppView::SliceView { face, idx } => {
        //         if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
        //             if idx > 0 {
        //                 self.nav.push(AppView::SliceView { face, idx: idx - 1 });
        //                 self.restart();
        //             }
        //         } else if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
        //             self.nav.push(AppView::SliceView { face, idx: idx + 1 });
        //             self.restart();
        //         }
        //     }
        //     _ => {}
        // };
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) {
            if rl.is_key_down(KeyboardKey::KEY_LEFT_ALT) {
                self.nav.go_back();
                self.restart();
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            if rl.is_key_down(KeyboardKey::KEY_LEFT_ALT) {
                self.nav.go_forward();
                self.restart();
            }
        }
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

    // fn draw_triangle(&self, d: &mut Option<&mut RaylibDrawHandle>, t: &Triangle, color: ColorType) {
    //     let Some(d) = d else {
    //         for line in t.lines() {
    //             self.draw_line(d, line.a, line.b, color);
    //         }
    //         return;
    //     };
    //     let t = self.bb.reproject_triangle(t);
    //     let a = self.to_canvas(t.a);
    //     let b = self.to_canvas(t.b);
    //     let c = self.to_canvas(t.c);
    //     let ab = a - b;
    //     let ac = a - c;
    //     let cross = ab.x * ac.y - ab.y * ac.x;
    //     // we need to sort to clockwise
    //     if let Some(fill) = color.fill() {
    //         match cross.signum() {
    //             -1.0 => d.draw_triangle(a, b, c, fill),
    //             _ => d.draw_triangle(a, c, b, fill),
    //         };
    //     }
    //     if let Some(stroke) = color.stroke() {
    //         match cross.signum() {
    //             -1.0 => d.draw_triangle_lines(a, b, c, stroke),
    //             _ => d.draw_triangle_lines(a, c, b, stroke),
    //         };
    //     }
    // }

    fn from_canvas(&self, p: &Point) -> Point {
        let p = self.nav.zoom.unproject(p);
        Point {
            x: ((p.x - 132.5) * 2.0 / 765.0 - 1.0),
            y: (-((p.y * 2.0 / 765.0) - 1.0)),
            z: 0.0,
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

    pub fn render(&mut self, r: &mut impl Renderer) {
        r.with_raylib(&mut |d| {
            d.clear_background(Color::WHITE);
        });
        let view = self.nav.current();
        match view {
            // AppView::SliceView { .. } => self.render_debug(d),
            _ => self.render_standard(r),
        };
        if let Some(selection) = self.selection {
            let pos = selection.0;
            let size = selection.1 - selection.0;
            r.with_raylib(&mut |d| {
                d.draw_rectangle_lines(
                    pos.x as i32,
                    pos.y as i32,
                    size.x as i32,
                    size.y as i32,
                    Color::RED,
                );
            });
        }
    }

    // pub fn render_debug(&self, r: &mut impl Renderer) {
    //     let Some(debug_view) = &self.debug_view else {
    //         self.render_standard(r);
    //         return;
    //     };
    //     for face in self.faces.iter() {
    //         if face.culled {
    //             continue;
    //         }
    //         for t in &face.haircut {
    //             self.draw_triangle(r, t, ColorType::Dark);
    //         }
    //     }
    //     let DebugView {
    //         tri,
    //         haircut,
    //         cutter,
    //     } = debug_view;
    //     self.draw_triangle(r, tri, ColorType::Lhs);
    //     self.draw_triangle(r, cutter, ColorType::Rhs);
    //     for cut in haircut {
    //         self.draw_triangle(r, cut, ColorType::Difference);
    //     }
    // }

    pub fn render_standard(&self, r: &mut impl Renderer) {
        let lights = [
            (
                ColorType::Pink,
                Point {
                    x: -2.0,
                    y: -3.0,
                    z: 1.9,
                }
                .normalize(),
            ),
            (
                ColorType::Blue,
                Point {
                    x: 0.2,
                    y: -1.5,
                    z: 2.0,
                }
                .normalize(),
            ),
        ];
        for (color, light) in lights {
            for face in self.faces.iter() {
                if face.culled {
                    continue;
                }
                for line in face.hatch(&light) {
                    r.draw_line(&line.a, &line.b, color);
                }
            }
        }
        /*
        for face in self.faces.iter() {
            if face.culled {
                continue;
            }
            for t in &face.haircut {
                if self.selected_faces.contains(&face.id) {
                    self.draw_triangle(d, t, ColorType::Selected);
                } else {
                    self.draw_triangle(
                        d,
                        t,
                        match face.haircut.len() {
                            _ => ColorType::Primary,
                            // _ => ColorType::Cut,
                        },
                    );
                }
            }
        }
        */
        // println!("SP{};", ColorType::Black.pen());
        for edge in self.edges.iter() {
            for cut_line in &edge.cut {
                r.draw_line(&cut_line.a, &cut_line.b, ColorType::Black);
            }
        }
    }

    pub fn clear(&self, d: &mut Option<&mut RaylibDrawHandle>) {
        let Some(d) = d else {
            return;
        };
        d.clear_background(Color::WHITE);
    }

    pub fn pointer_click(&mut self, x: f32, y: f32) {
        eprintln!("Clicked: {:?}", self.selected_faces);
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
            x: (x as f64),
            y: (y as f64),
            z: 0.0,
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
        let mut cull_count = 0;
        for face in self.faces.iter_mut() {
            let n = face.calc_normal();
            let c = face.calc_centroid().normalize();
            let which_way = n.dot(&c);
            if which_way <= 0.0 {
                face.culled = true;
                cull_count += 1;
            }
        }
        eprintln!("culled {} backfaces", cull_count);
        for comb in self.faces.iter().combinations(2) {
            if comb[0].culled == comb[1].culled {
                continue;
            }
            let shared_lines: Vec<_> = comb[0]
                .hair
                .lines()
                .filter(|l| comb[1].hair.has_line(*l))
                .collect();
            let shared_line = match shared_lines[..] {
                [line] => line,
                [] => continue,
                _ => panic!("two faces share more than one line??"),
            };
            self.edges.push(Edge {
                line: shared_line,
                face_ids: vec![comb[0].id, comb[1].id],
                cut: vec![],
            });
        }
        eprintln!("found {} edges", self.edges.len());
    }

    pub fn partial_culling(&mut self) {
        // it's time to split hairs
        let mut drawn: Vec<Vec<Vec<Point>>> = vec![vec![]];
        let mut face_count = 0;
        for face in self.faces.iter_mut() {
            if face.culled {
                continue;
            }
            let hair_clip = vec![face.hair.a, face.hair.b, face.hair.c];
            let clip = hair_clip.overlay(&drawn, OverlayRule::Difference, FillRule::EvenOdd);
            if clip.is_empty() {
                face.culled = true;
                continue;
            }
            let clap = clip.triangulate().to_triangulation::<usize>();
            let points: Vec<_> = clap.indices.iter().map(|&i| clap.points[i]).collect();
            face.haircut = points
                .chunks_exact(3)
                .filter_map(|set| match set {
                    [a, b, c] => Some(Triangle {
                        a: *a,
                        b: *b,
                        c: *c,
                    }),
                    _ => None,
                })
                .collect();
            for face_edge in self
                .edges
                .iter_mut()
                .filter(|e| e.face_ids.contains(&face.id))
            {
                let cut = [face_edge.line.a, face_edge.line.b].clip_by(
                    &drawn,
                    FillRule::EvenOdd,
                    ClipRule {
                        invert: true,
                        boundary_included: true,
                    },
                );
                face_edge.cut = cut.into_iter().map(|l| Line { a: l[0], b: l[1] }).collect();
            }
            // drawn.push(vec![hair_clip]);
            // drawn = drawn.simplify_shape(FillRule::EvenOdd);
            drawn = hair_clip.overlay(&drawn, OverlayRule::Union, FillRule::EvenOdd);
            face_count += 1;
            // eprintln!("processed {} faces", face_count);
        }
    }

    pub fn restart(&mut self) {
        // console::log_1(&format!("view: {:?}", self.view));
        self.edges.clear();
        self.faces.clear();
        self.debug_view = None;
        let mut v: Vec<Point> = vec![];
        let mut vn: Vec<Point> = vec![];
        let theta_y: f64 = std::f64::consts::PI / 2.0 * 1.3;
        let theta_x: f64 = std::f64::consts::PI / 2.0 * 0.1;
        for line in TEAPOT.lines() {
            let parts = line.split(" ").collect::<Vec<_>>();
            match parts[0] {
                "f" => {
                    let parts = parts
                        .iter()
                        .skip(1)
                        .map(|p| {
                            let parts = p.split("/").collect::<Vec<_>>();
                            let vertex = v[parts[0].parse::<usize>().unwrap() - 1]
                                .rotate_y(theta_y)
                                .rotate_x(theta_x);
                            let vertex = translate(vertex);
                            FacePart {
                                vertex,
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

        eprintln!(
            "parsed {} faces, {} vertices, and {} normals",
            self.faces.len(),
            v.len(),
            vn.len()
        );

        let _count = 0;
        self.faces.sort();
        for (z, face) in self.faces.iter_mut().enumerate() {
            face.id = z;
        }

        self.backface_culling();
        self.partial_culling();
        self.bb.make_square();
        self.find_contours();
    }
    pub fn find_contours(&mut self) {
        let mut subj: Vec<Vec<Vec<Point>>> = vec![vec![]];
        self.contours.clear();
        for face in self.faces.iter() {
            let t = face.hair;
            let clip = [t.a, t.b, t.c];
            let result = subj.overlay(&clip, OverlayRule::Union, FillRule::EvenOdd);
            subj = result;
        }
        for i in 0..=100 {
            let z = (i as f64) / 20.0 - 2.0;
            let plane = Plane {
                point: Point {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                offset: z,
            };
            for face in self.faces.iter() {
                if face.culled {
                    continue;
                }
                let clip_rule = ClipRule {
                    invert: false,
                    boundary_included: false,
                };
                let res = face.as_triangle().plane_intersection(&plane);
                match res.len() {
                    0 => {}
                    1 => {}
                    2 => {
                        let mut subj: Vec<Vec<Vec<Point>>> = vec![vec![]];
                        // join the haircut into a clip mask
                        for t in face.haircut.iter() {
                            let clip = [t.a, t.b, t.c];
                            let result = subj.overlay(&clip, OverlayRule::Union, FillRule::EvenOdd);
                            subj = result;
                        }
                        let line = [project(res[0]), project(res[1])];
                        for shape in subj {
                            let result = line.clip_by(&shape, FillRule::NonZero, clip_rule);
                            for line in result {
                                self.contours.push(Line {
                                    a: line[0],
                                    b: line[1],
                                });
                            }
                        }
                    }
                    _ => unimplemented!("wtf"),
                }
            }
        }
    }
}
