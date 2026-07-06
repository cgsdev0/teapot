// fn cursed_subtraction_debug(
//     t: &Triangle,
//     cutter: &Triangle,
//     i: &Vec<Intersection>,
//     want_panic: bool,
// ) {
//     return;
//     if !EVIL_MODE.load(Ordering::Relaxed) {
//         return;
//     }
//     let skip = SKIP.load(Ordering::Relaxed);
//     if skip < 4 && !want_panic {
//         SKIP.store(skip + 1, Ordering::Relaxed);
//         return;
//     }
//     if want_panic {
//         if DONT_RECURSE.load(Ordering::Relaxed) {
//             return;
//         }
//         DONT_RECURSE.store(true, Ordering::Relaxed);
//     } else {
//         if DONT_RECURSE2.load(Ordering::Relaxed) {
//             return;
//         }
//         DONT_RECURSE2.store(true, Ordering::Relaxed);
//     }
//     eprintln!("cursed entry");
//     let mut bb = BoundingBox::new();
//     bb.expand(&t.a);
//     bb.expand(&t.b);
//     bb.expand(&t.c);
//     // bb.expand(&cutter.a);
//     // bb.expand(&cutter.b);
//     // bb.expand(&cutter.c);
//     let t2 = bb.reproject_triangle(t);
//     let c2 = bb.reproject_triangle(cutter);
//     let draw = |tris: Vec<Triangle>| {
//         eprintln!("{:?}", t);
//         println!("const bounding_boxes = null;");
//         println!("const plot = async () => {{");

//         bb.draw();
//         draw_triangle_js(&t2, Color::Lhs);
//         draw_triangle_js(&c2, Color::Rhs);
//         let mut points: HashMap<Point, usize> = std::collections::HashMap::new();
//         for point in [t2.a, t2.b, t2.c, c2.a, c2.b, c2.c] {
//             match points.get(&point) {
//                 Some(count) => {
//                     points.insert(point, count + 1);
//                 }
//                 None => {
//                     points.insert(point, 1);
//                 }
//             }
//         }
//         for (point, count) in points.iter() {
//             draw_point_js(
//                 *point,
//                 match count {
//                     1 => "magenta",
//                     _ => "yellow",
//                 },
//                 false,
//             );
//         }
//         for aye in i
//             .iter()
//             .filter(|i| i.real && t.points().all(|p| i.point.dist2(&p) > f64::EPSILON.into()))
//         {
//             let eye = bb.reproject(&aye.point);
//             if !aye.projected && !aye.real {
//                 continue;
//             }
//             draw_point_js(
//                 eye,
//                 if aye.projected { "#33aaff" } else { "#fff" },
//                 !aye.real,
//             );
//         }
//         for t in tris {
//             draw_triangle_js(&bb.reproject_triangle(&t), Color::Difference);
//         }
//         println!("}}");
//     };
//     if want_panic {
//         let res = std::panic::catch_unwind(|| {
//             // we don't care about the result;
//             // either this panics and we draw,
//             // or it doesn't panic and we don't care anyways.
//             let _ = *t - *cutter;
//         });
//         match res {
//             Err(_) => {
//                 eprintln!("PANIC CAPTURED");
//                 draw(vec![]);
//             }
//             Ok(_) => {
//                 // no panic? lame
//                 DONT_RECURSE.store(false, Ordering::Relaxed);
//                 return;
//             }
//         }
//     } else {
//         let split = *t - *cutter;
//         // eprintln!("{} splits", split.len());
//         draw(split);
//     }
//     // eprintln!("{:?}\n\n{:?}\n\n{:?}", bb, t, cutter);
//     std::process::exit(0);
// }
