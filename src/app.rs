use crate::bounding_box::BoundingBox;
use crate::renderer::ColorType;
use i_overlay::core::fill_rule::FillRule;
use i_overlay::core::overlay_rule::OverlayRule;
use i_overlay::float::clip::FloatClip;
use i_overlay::float::single::SingleFloatOverlay;
use i_overlay::string::clip::ClipRule;
use i_triangle::float::triangulatable::Triangulatable;
use imgui::Context;
use itertools::Itertools;
use nalgebra::{Perspective3, Rotation3};
use ordered_float::OrderedFloat;
use raylib::prelude::*;
use rfd::FileDialog;
use std::collections::{HashMap, HashSet};
use std::ops::Mul;
use std::sync::mpsc::{Receiver, Sender};
extern crate nalgebra as na;
use na::Vector3;

use raylib::prelude::RaylibDrawHandle;

use crate::geometry::*;
use crate::navigator::*;
use crate::renderer::*;

const TEAPOT: &str = include_str!("../models/dragon.obj");

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
}

pub type Haircut = Vec<Triangle>;

impl PartialOrd for Face {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Face {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
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
    pub fn light_normal(&self) -> Point {
        let mut out: Point = self.calc_normal();
        if let Some(a) = self.eyes.normal {
            if let Some(b) = self.noes.normal {
                if let Some(c) = self.ears.normal {
                    out = (a + b + c).normalize();
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct DebugView {
    pub tri: Triangle,
    pub haircut: Vec<Triangle>,
    pub cutter: Triangle,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub line: Line,
    pub face_ids: Vec<usize>,
    pub cut: Vec<Line>,
}

#[derive(Debug, Clone)]
pub struct Contour {
    pub line: Line,
    pub face_id: usize,
    pub cut: Vec<Line>,
}

#[derive(Default, Clone, Debug)]
pub struct Render {
    pub edges: Vec<Edge>,
    pub shape: Vec<Vec<Point>>,
    pub culled: HashMap<usize, bool>,
    pub haircuts: HashMap<usize, Haircut>,
}

impl Render {
    pub fn is_culled(&self, face: &Face) -> bool {
        self.culled.get(&face.id).is_some_and(|x| *x)
    }

    pub fn get_haircut(&self, face: &Face) -> &Haircut {
        self.haircuts.get(&face.id).unwrap()
    }

    pub fn backface_culling(&mut self, model: &Model) {
        let mut cull_count = 0;
        for face in model.faces.iter() {
            let n = face.calc_normal();
            let c = face.calc_centroid().normalize();
            let which_way = n.dot(&c);
            if which_way <= 0.0 {
                self.culled.insert(face.id, true);
                cull_count += 1;
            }
        }
        eprintln!("culled {} backfaces", cull_count);
        for comb in model.faces.iter().combinations(2) {
            if self.is_culled(comb[0]) == self.is_culled(comb[1]) {
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

    pub fn partial_culling(&mut self, model: &Model) {
        // it's time to split hairs
        let mut drawn: Vec<Vec<Vec<Point>>> = vec![vec![]];
        let mut face_count = 0;
        for face in model.faces.iter() {
            if self.is_culled(face) {
                continue;
            }
            let hair_clip = vec![face.hair.a, face.hair.b, face.hair.c];
            let clip = hair_clip.overlay(&drawn, OverlayRule::Difference, FillRule::EvenOdd);
            if clip.is_empty() {
                self.culled.insert(face.id, true);
                continue;
            }
            let clap = clip.triangulate().to_triangulation::<usize>();
            let points: Vec<_> = clap.indices.iter().map(|&i| clap.points[i]).collect();
            self.haircuts.insert(
                face.id,
                points
                    .chunks_exact(3)
                    .filter_map(|set| match set {
                        [a, b, c] => Some(Triangle {
                            a: *a,
                            b: *b,
                            c: *c,
                        }),
                        _ => None,
                    })
                    .collect(),
            );
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
}

#[derive(Default, Clone, Debug)]
pub struct Lighting {
    pub longitude_contours: Vec<Contour>,
    pub latitude_contours: Vec<Contour>,
    pub shadowed: HashMap<usize, f64>,
}

impl Lighting {
    pub fn shadow(&self, face: &Face) -> f64 {
        self.shadowed.get(&face.id).copied().unwrap_or(1.0)
    }
    pub fn lightmap(&mut self, light: &Point, model: &Model, render: &Render) {
        let faces = model.faces.clone();
        let light2 = -1.0 * (*light);
        for face in model.faces.iter() {
            if render.is_culled(face) {
                continue;
            }
            let mut hit: Option<f64> = None;
            for f in faces.iter() {
                if f.id == face.id {
                    continue;
                }
                if raycast(&f.as_triangle(), &face.calc_centroid(), &light2) {
                    let n = face.calc_normal().dot(&f.calc_normal());
                    if let Some(h) = hit {
                        hit = Some(h.min(n));
                    } else {
                        hit = Some(n);
                    }
                }
            }
            if let Some(hit) = hit {
                self.shadowed.insert(face.id, hit);
            }
        }
    }

    pub fn find_longitude_contours(&mut self, light: &Point, model: &Model, render: &Render) {
        let mut subj: Vec<Vec<Vec<Point>>> = vec![vec![]];
        for face in model.faces.iter() {
            let t = face.hair;
            let clip = [t.a, t.b, t.c];
            let result = subj.overlay(&clip, OverlayRule::Union, FillRule::EvenOdd);
            subj = result;
        }
        let light = light.normalize();
        for i in 0..=120 {
            let theta = (i as f64) / 120.0 * 2.0 * std::f64::consts::PI;
            let lx = light.x.abs();
            let ly = light.y.abs();
            let lz = light.z.abs();
            let garbage = if lx <= ly && lx <= lz {
                Point {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                }
            } else if ly <= lz {
                Point {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                }
            } else {
                Point {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }
            };

            let u = light.cross(&garbage).normalize();
            let w = light.cross(&u).normalize();
            let normal = u * theta.cos() + w * theta.sin();

            let origin = -1.0
                * translate(
                    Point {
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    &model.model_bb,
                );
            let z = normal.dot(&origin);
            let plane = Plane {
                point: normal,
                offset: z,
            };
            for face in model.faces.iter() {
                if render.is_culled(face) {
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
                    4 | 3 | 2 => {
                        let line = [project(res[0]), project(res[1])];
                        let mut contour = Contour {
                            line: Line {
                                a: line[0],
                                b: line[1],
                            },
                            face_id: face.id,
                            cut: vec![],
                        };
                        let mut subj: Vec<Vec<Vec<Point>>> = vec![vec![]];
                        // join the haircut into a clip mask
                        for t in render.get_haircut(face).iter() {
                            let clip = [t.a, t.b, t.c];
                            let result = subj.overlay(&clip, OverlayRule::Union, FillRule::EvenOdd);
                            subj = result;
                        }
                        for shape in subj {
                            let result = line.clip_by(&shape, FillRule::NonZero, clip_rule);
                            for line in result {
                                contour.cut.push(Line {
                                    a: line[0],
                                    b: line[1],
                                });
                            }
                        }
                        if !contour.cut.is_empty() {
                            self.longitude_contours.push(contour);
                        }
                    }
                    n => unimplemented!("wtf {}", n),
                }
            }
        }
    }
    pub fn find_latitude_contours(&mut self, light: &Point, model: &Model, render: &Render) {
        let mut subj: Vec<Vec<Vec<Point>>> = vec![vec![]];
        for face in model.faces.iter() {
            let t = face.hair;
            let clip = [t.a, t.b, t.c];
            let result = subj.overlay(&clip, OverlayRule::Union, FillRule::EvenOdd);
            subj = result;
        }
        let light = light.normalize();
        for i in 0..=800 {
            let z = (i as f64) / 20.0 - 4.0;
            let plane = Plane {
                point: light,
                offset: z,
            };
            for face in model.faces.iter() {
                if render.is_culled(face) {
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
                    3 => {
                        // TODO: this should be some kinda line i think
                    }
                    4 => {
                        // we are in floating point hell
                    }
                    2 => {
                        let line = [project(res[0]), project(res[1])];
                        let mut contour = Contour {
                            line: Line {
                                a: line[0],
                                b: line[1],
                            },
                            face_id: face.id,
                            cut: vec![],
                        };
                        let mut subj: Vec<Vec<Vec<Point>>> = vec![vec![]];
                        // join the haircut into a clip mask
                        for t in render.get_haircut(face).iter() {
                            let clip = [t.a, t.b, t.c];
                            let result = subj.overlay(&clip, OverlayRule::Union, FillRule::EvenOdd);
                            subj = result;
                        }
                        for shape in subj {
                            let result = line.clip_by(&shape, FillRule::NonZero, clip_rule);
                            for line in result {
                                contour.cut.push(Line {
                                    a: line[0],
                                    b: line[1],
                                });
                            }
                        }
                        if !contour.cut.is_empty() {
                            self.latitude_contours.push(contour);
                        }
                    }
                    n => unimplemented!("wtf {}", n),
                }
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Model {
    pub faces: Vec<Face>,
    pub screen_bb: BoundingBox,
    pub model_bb: BoundingBox,
}

impl Model {
    pub fn re_scale_model(&mut self) {
        let scale_options = [
            (self.model_bb.max.x - self.model_bb.min.x),
            (self.model_bb.max.y - self.model_bb.min.y),
            // (self.model_bb.max.z - self.model_bb.min.z),
        ];
        let scale = scale_options
            .iter()
            .copied()
            .map(OrderedFloat::from)
            .max()
            .unwrap();
        let scale = 2.0 / (scale.into_inner());
        eprintln!("scale: {}", scale);
        self.model_bb.max = self.model_bb.max * scale;
        self.model_bb.min = self.model_bb.min * scale;
        for face in self.faces.iter_mut() {
            face.eyes.vertex = face.eyes.vertex * scale;
            face.noes.vertex = face.noes.vertex * scale;
            face.ears.vertex = face.ears.vertex * scale;
            // re-calc screen_bb
        }
    }
}

pub struct ProgressMessage {
    kind: ProgressType,
    render_id: u64,
}

#[derive(Debug)]
pub enum ProgressType {
    Model(Model),
    Render(Render),
    Lighting(Lighting),
}

pub struct AppState {
    pub selected_faces: HashSet<usize>,
    pub nav: Navigator,
    pub debug_view: Option<DebugView>,
    pub selection: Option<(Vector2, Vector2)>,
    pub light: Point,
    pub model_data: String,
    pub model: Option<Model>,
    pub render: Option<Render>,
    pub lighting: Option<Lighting>,
    pub channel: Option<Receiver<ProgressMessage>>,
    pub render_id: u64,
    pub blocking_messages: bool,
    pub euler_angles: (f64, f64, f64),
    pub thresholds: (f64, f64, f64, f64),
}

pub fn raycast(t: &Triangle, start: &Point, dir: &Point) -> bool {
    let e1 = t.b - t.a;
    let e2 = t.c - t.a;

    let pvec = dir.cross(&e2);
    let det = e1.dot(&pvec);

    if det.abs() < 1e-8 {
        return false;
    }

    let inv_det = 1.0 / det;
    let tvec = *start - t.a;

    let u = tvec.dot(&pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return false;
    }

    let qvec = tvec.cross(&e1);
    let w = dir.dot(&qvec) * inv_det;
    if w < 0.0 || u + w > 1.0 {
        return false;
    }

    let t = e2.dot(&qvec) * inv_det;
    t >= 1e-6 && t <= f64::INFINITY
}

fn transform(p: &Point, euler_angles: &(f64, f64, f64), model_bb: &BoundingBox) -> Point {
    let mat = Rotation3::from_euler_angles(
        euler_angles.0 / 180.0 * PI,
        euler_angles.1 / 180.0 * PI,
        euler_angles.2 / 180.0 * PI,
    );
    let res = mat.mul(&Vector3::new(p.x, p.y, p.z));
    translate(
        Point {
            x: res.x,
            y: res.y,
            z: res.z,
        },
        model_bb,
    )
}

fn project_tri(t: &Triangle) -> Triangle {
    Triangle {
        a: project(t.a),
        b: project(t.b),
        c: project(t.c),
    }
}

fn project(p: Point) -> Point {
    let mat = Perspective3::new(1.0, 0.5, 1.0, 100.0);
    let res = mat.project_vector(&Vector3::new(p.x, -p.y, p.z));
    Point {
        x: res.x,
        y: res.y,
        z: res.z,
    }
}

fn fit_distance() -> f64 {
    let radius = (2.0_f64).sqrt();
    let aspect = 1.0;
    let fov: f64 = 0.5;
    let dist_v = radius / (fov / 2.0).tan();
    let dist_h = radius / ((fov / 2.0).tan() * aspect);
    [dist_v, dist_h]
        .iter()
        .copied()
        .map(OrderedFloat)
        .max()
        .unwrap()
        .into_inner()
}

fn translate(p: Point, bb: &BoundingBox) -> Point {
    let dx = (bb.max.x + bb.min.x) / 2.0;
    let dy = (bb.max.y + bb.min.y) / 2.0;
    Point {
        x: p.x - dx,
        y: p.y - dy,
        z: p.z + fit_distance(),
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
            nav: Navigator::new(),
            selected_faces: HashSet::new(),
            debug_view: None,
            selection: None,
            light: Point {
                x: 0.5,
                y: -1.0,
                z: 0.5,
            },
            model_data: TEAPOT.to_string(),
            model: None,
            render: None,
            lighting: None,
            channel: None,
            render_id: 0,
            blocking_messages: false,
            euler_angles: (0.0, 0.0, 0.0),
            thresholds: (0.3, 0.3, -0.5, 0.6),
        }
    }
    pub fn transform_face(&self, model_bb: &BoundingBox, f: &Face) -> Face {
        Face {
            eyes: FacePart {
                vertex: transform(&f.eyes.vertex, &self.euler_angles, &model_bb),
                normal: f.eyes.normal,
            },
            noes: FacePart {
                vertex: transform(&f.noes.vertex, &self.euler_angles, &model_bb),
                normal: f.noes.normal,
            },
            ears: FacePart {
                vertex: transform(&f.ears.vertex, &self.euler_angles, &model_bb),
                normal: f.ears.normal,
            },
            ..(*f)
        }
    }
    pub fn update(&mut self, rl: &mut RaylibHandle, imgui: &mut Context) {
        self.nav.zoom.update(rl.get_frame_time() as f64);
        if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
            let mut r = HpglRenderer::new();
            self.render(&mut r, imgui);
        }

        // mouse stuff (squeak squeak)
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
                    // self.pointer_click(pos.x, pos.y);
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

    pub fn render(&mut self, r: &mut impl Renderer, imgui: &mut Context) {
        // DO UI STUFF
        r.with_raylib(&mut |_| {
            if let Some(channel) = &self.channel {
                for msg in channel.try_iter() {
                    if self.blocking_messages {
                        continue;
                    }
                    if msg.render_id < self.render_id {
                        continue;
                    }
                    match msg.kind {
                        ProgressType::Model(model) => {
                            println!("got model msg");
                            self.model = Some(model);
                        }
                        ProgressType::Render(render) => {
                            println!("got render msg");
                            self.render = Some(render);
                        }
                        ProgressType::Lighting(lighting) => {
                            println!("got lighting msg");
                            self.lighting = Some(lighting);
                        }
                    }
                }
            }
            let ui = imgui.new_frame();
            if let Some(_t) = ui.begin_main_menu_bar() {
                if let Some(_t) = ui.begin_menu("File") {
                    if ui.menu_item("Open") {
                        let fname = FileDialog::new()
                            .add_filter("Wavefront .obj", &["obj"])
                            .set_directory("./models")
                            .pick_file();
                        if let Some(fname) = fname {
                            self.model_data = std::fs::read_to_string(&fname).unwrap();
                            self.restart(true);
                        }
                    }
                }
            }
            if let Some(_t) = ui.window("Model").begin() {
                let ax = ui.slider("X", 0.0, 360.0, &mut self.euler_angles.0);
                let x = ui.is_item_deactivated_after_edit();
                let ay = ui.slider("Y", 0.0, 360.0, &mut self.euler_angles.1);
                let y = ui.is_item_deactivated_after_edit();
                let az = ui.slider("Z", 0.0, 360.0, &mut self.euler_angles.2);
                let z = ui.is_item_deactivated_after_edit();
                if ax || ay || az {
                    self.blocking_messages = true;
                    self.render = None;
                    self.lighting = None;
                }
                if x || y || z {
                    self.blocking_messages = false;
                    self.restart(false);
                }
            }
            if let Some(_t) = ui.window("Light").begin() {
                let ax = ui.slider("X", -5.0, 5.0, &mut self.light.x);
                let x = ui.is_item_deactivated_after_edit();
                let ay = ui.slider("Y", -5.0, 5.0, &mut self.light.y);
                let y = ui.is_item_deactivated_after_edit();
                let az = ui.slider("Z", -5.0, 5.0, &mut self.light.z);
                let z = ui.is_item_deactivated_after_edit();
                if ax || ay || az {
                    self.blocking_messages = true;
                    self.render = None;
                    self.lighting = None;
                }
                if x || y || z {
                    self.blocking_messages = false;
                    self.restart(false);
                }
            }
            if let Some(_t) = ui.window("Hatching").begin() {
                ui.slider("A", -1.0, 1.0, &mut self.thresholds.0);
                ui.slider("B", -1.0, 1.0, &mut self.thresholds.1);
                ui.label_text("", "Shadows");
                ui.slider("C", -1.0, 1.0, &mut self.thresholds.2);
                ui.slider("D", -1.0, 1.0, &mut self.thresholds.3);
            }
        });

        // NOW RENDER

        r.with_raylib(&mut |d| {
            d.clear_background(Color::WHITE);
            d.draw_fps(15, 25);
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

    pub fn render_standard(&self, r: &mut impl Renderer) {
        match (&self.model, &self.render, &self.lighting) {
            (None, None, None) => {
                // spinner?
            }
            // Phase 1
            (Some(model), None, None) => {
                r.with_raylib(&mut |d| {
                    d.clear_background(Color::GRAY);
                });
                for face in model
                    .faces
                    .iter()
                    .map(|f| self.transform_face(&model.model_bb, &f))
                    .sorted()
                    .rev()
                {
                    let hair = project_tri(&face.as_triangle());
                    let normal = face.calc_normal();
                    let light = (normal.dot(&self.light) / 2.0 + 0.5) * 255.0;
                    r.draw_triangle(&hair, ColorType::Shaded(light as u8));
                }
            }
            // Phase 2
            (Some(model), Some(render), None) => {
                r.with_raylib(&mut |d| {
                    d.clear_background(Color::GRAY);
                });
                for face in model.faces.iter() {
                    if render.is_culled(face) {
                        continue;
                    }
                    if let Some(haircut) = render.haircuts.get(&face.id) {
                        // let normal = face.calc_normal();
                        // let light = (normal.dot(&self.light) / 2.0 + 0.5) * 255.0;
                        for tri in haircut.iter() {
                            r.draw_triangle(tri, ColorType::Pink);
                        }
                    }
                }
            }
            // Phase 3
            (Some(model), Some(render), Some(lighting)) => {
                for contour in lighting.longitude_contours.iter() {
                    let face = self.transform_face(&model.model_bb, &model.faces[contour.face_id]);
                    let normal = face.calc_normal();
                    let dot = self.light.normalize().dot(&normal);
                    if dot > self.thresholds.0 && lighting.shadow(&face) > self.thresholds.2 {
                        continue;
                    }
                    for line in contour.cut.iter() {
                        r.draw_line(&line.a, &line.b, ColorType::Contour);
                    }
                }
                for contour in lighting.latitude_contours.iter() {
                    let face = self.transform_face(&model.model_bb, &model.faces[contour.face_id]);
                    let normal = face.calc_normal();
                    let dot = self.light.normalize().dot(&normal);
                    if dot > self.thresholds.1 && lighting.shadow(&face) > self.thresholds.3 {
                        continue;
                    }
                    for line in contour.cut.iter() {
                        r.draw_line(&line.a, &line.b, ColorType::Contour);
                    }
                }
                for edge in render.edges.iter() {
                    for cut_line in &edge.cut {
                        r.draw_line(&cut_line.a, &cut_line.b, ColorType::Outline);
                    }
                }
            }
            _ => {
                unreachable!("why would you do this to me")
            }
        }
    }

    pub fn pointer_move(&mut self, x: f32, y: f32) {
        // let p = Point {
        //     x: (x as f64),
        //     y: (y as f64),
        //     z: 0.0,
        // };
        // let p = self.from_canvas(&p);
        // let p = self.screen_bb.unproject(&p);
        // let mut dirty = false;
        // for (i, face) in self.faces.iter().enumerate() {
        //     if face.culled {
        //         continue;
        //     }
        //     for t in face.haircut.iter() {
        //         if t.contains(&p) {
        //             if !dirty {
        //                 dirty = true;
        //                 self.selected_faces.clear();
        //             }
        //             self.selected_faces.insert(face.id);
        //             break;
        //         }
        //     }
        // }
        // if !dirty && !self.selected_faces.is_empty() {
        //     self.selected_faces.clear();
        // }
    }

    pub fn restart(&mut self, load_model: bool) {
        self.render_id += 1;
        let (mut sender, receiver) = std::sync::mpsc::channel::<ProgressMessage>();
        self.lighting = None;
        self.render = None;
        if load_model {
            self.model = None;
        }
        let model2 = self.model.clone();
        let render_id = self.render_id;
        let model_data = self.model_data.clone();
        let light = self.light;
        let euler_angles = self.euler_angles;
        self.channel = Some(receiver);
        std::thread::spawn(move || {
            let mut model = if load_model {
                let mut model = Model::default();
                let mut v: Vec<Point> = vec![];
                let mut vn: Vec<Point> = vec![];
                for line in model_data.lines() {
                    let parts = line.split(" ").collect::<Vec<_>>();
                    match parts[0] {
                        "f" => {
                            let parts = parts
                                .iter()
                                .skip(1)
                                .map(|p| {
                                    let parts = p.split("/").collect::<Vec<_>>();
                                    let vertex = v[parts[0].parse::<usize>().unwrap() - 1];
                                    // let vertex = translate(vertex);
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
                            };
                            model.model_bb.expand(&face.eyes.vertex);
                            model.model_bb.expand(&face.noes.vertex);
                            model.model_bb.expand(&face.ears.vertex);
                            model.faces.push(face);
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

                eprintln!(
                    "parsed {} faces, {} vertices, and {} normals",
                    model.faces.len(),
                    v.len(),
                    vn.len()
                );

                let _count = 0;
                model.re_scale_model();
                for (z, face) in model.faces.iter_mut().enumerate() {
                    face.id = z;
                }

                {
                    // progress report!
                    let model = model.clone();
                    sender
                        .send(ProgressMessage {
                            kind: ProgressType::Model(model),
                            render_id,
                        })
                        .unwrap();
                }
                model
            } else {
                model2.unwrap()
            };

            for face in model.faces.iter_mut() {
                *face = Face {
                    eyes: FacePart {
                        vertex: transform(&face.eyes.vertex, &euler_angles, &model.model_bb),
                        normal: face.eyes.normal,
                    },
                    noes: FacePart {
                        vertex: transform(&face.noes.vertex, &euler_angles, &model.model_bb),
                        normal: face.noes.normal,
                    },
                    ears: FacePart {
                        vertex: transform(&face.ears.vertex, &euler_angles, &model.model_bb),
                        normal: face.ears.normal,
                    },
                    id: face.id,
                    hair: face.hair,
                };
                face.hair = Triangle {
                    a: project(face.eyes.vertex),
                    b: project(face.noes.vertex),
                    c: project(face.ears.vertex),
                };
            }
            model.faces.sort();

            let mut render = Render::default();
            render.backface_culling(&model);
            render.partial_culling(&model);
            {
                // progress report!
                let render = render.clone();
                sender
                    .send(ProgressMessage {
                        kind: ProgressType::Render(render),
                        render_id,
                    })
                    .unwrap();
            }

            let mut lighting = Lighting::default();
            lighting.find_latitude_contours(&light, &model, &render);
            lighting.find_longitude_contours(&light, &model, &render);
            lighting.lightmap(&light, &model, &render);
            {
                // progress report!
                sender
                    .send(ProgressMessage {
                        kind: ProgressType::Lighting(lighting),
                        render_id,
                    })
                    .unwrap();
            }
        });
    }
}
