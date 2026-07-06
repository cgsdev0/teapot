use crate::app::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

mod app;
mod geometry;

#[component]
fn App() -> Html {
    let canvas_ref = use_node_ref();

    // let selecting = use_mut_ref(|| false);
    let selection = use_mut_ref::<Option<(i32, i32, i32, i32)>, _>(|| None);

    let zoomed_in = use_state(|| false);

    let app = use_mut_ref(AppState::new);
    {
        let canvas_ref = canvas_ref.clone();
        let app = app.clone();
        let selection = selection.clone();
        use_effect_with(canvas_ref, move |canvas_ref| {
            let canvas = canvas_ref
                .cast::<HtmlCanvasElement>()
                .expect("canvas_ref not attached to canvas element");

            let ctx = canvas
                .get_context("2d")
                .expect("could not get context")
                .expect("could not get context");

            let ctx = ctx
                .dyn_into::<CanvasRenderingContext2d>()
                .expect("Failed to cast to CanvasRenderingContext2d");

            let pointerdown = Closure::<dyn Fn(Event)>::new(move |e: Event| {
                let e = e.dyn_into::<PointerEvent>().unwrap();
                let offset_x = e.offset_x();
                let offset_y = e.offset_y();
                let mut selection = selection.borrow_mut();
                *selection = Some((offset_x, offset_y, i32::MAX, i32::MAX));
            });

            canvas
                .add_event_listener_with_callback(
                    "pointerdown",
                    pointerdown.as_ref().unchecked_ref(),
                )
                .unwrap();

            canvas
                .add_event_listener_with_callback(
                    "pointerdown",
                    pointerdown.as_ref().unchecked_ref(),
                )
                .unwrap();

            // start the app and render
            app.borrow_mut().start(ctx);
            app.borrow().render();

            // effects return a cleanup method
            move || {
                canvas
                    .remove_event_listener_with_callback(
                        "pointerdown",
                        pointerdown.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        });
    }
    {
        let canvas_ref = canvas_ref.clone();
        let app = app.clone();
        let selection = selection.clone();
        use_effect_with(canvas_ref, move |canvas_ref| {
            let canvas = canvas_ref
                .cast::<HtmlCanvasElement>()
                .expect("canvas_ref not attached to canvas element");

            let pointermove = Closure::<dyn Fn(Event)>::new(move |e: Event| {
                let e = e.dyn_into::<PointerEvent>().unwrap();
                let offset_x = e.offset_x();
                let offset_y = e.offset_y();
                let mut selection = selection.borrow_mut();
                match &*selection {
                    Some(s) => {
                        let (sx, sy, _, _) = *s;
                        if (sx - offset_x).abs() <= 5 && (sy - offset_y).abs() <= 5 {
                            // small drag, ignore
                            app.borrow_mut().move_pointer(offset_x, offset_y);
                            return;
                        }
                        *selection = Some((sx, sy, offset_x, offset_y));
                        let app = app.borrow();
                        app.render();
                        let ctx = app.ctx.as_ref().unwrap();
                        ctx.set_stroke_style_str("magenta");
                        ctx.stroke_rect(
                            sx.into(),
                            sy.into(),
                            (offset_x - sx).into(),
                            (offset_y - sy).into(),
                        );
                    }
                    None => {
                        app.borrow_mut().move_pointer(offset_x, offset_y);
                    }
                };
            });

            canvas
                .add_event_listener_with_callback(
                    "pointermove",
                    pointermove.as_ref().unchecked_ref(),
                )
                .unwrap();

            // effects return a cleanup method
            move || {
                canvas
                    .remove_event_listener_with_callback(
                        "pointermove",
                        pointermove.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        });
    }
    {
        let zoomed_in = zoomed_in.clone();
        let canvas_ref = canvas_ref.clone();
        let app = app.clone();
        let selection = selection.clone();
        use_effect_with(canvas_ref, move |_canvas_ref| {
            let pointerup = Closure::<dyn Fn(Event)>::new(move |_e: Event| {
                let mut selection = selection.borrow_mut();
                match &*selection {
                    Some(s) if s.2 == i32::MAX || s.3 == i32::MAX => {
                        // insufficient drag distance; this is just a click
                        *selection = None;
                    }
                    Some(s) => {
                        let (sx, sy, tx, ty) = *s;
                        let mut app = app.borrow_mut();
                        zoomed_in.set(true);
                        app.zoom_to(sx as f64, sy as f64, tx as f64, ty as f64);
                        app.render();
                        *selection = None;
                    }
                    None => {}
                };
            });

            window()
                .unwrap()
                .add_event_listener_with_callback("pointerup", pointerup.as_ref().unchecked_ref())
                .unwrap();

            // effects return a cleanup method
            move || {
                window()
                    .unwrap()
                    .remove_event_listener_with_callback(
                        "pointerup",
                        pointerup.as_ref().unchecked_ref(),
                    )
                    .unwrap();
            }
        });
    }

    let reset_zoom = {
        let zoomed_in = zoomed_in.clone();
        let app = app.clone();
        Callback::from(move |_| {
            let mut app = app.borrow_mut();
            zoomed_in.set(false);
            app.reset_zoom();
            app.render();
        })
    };
    html! {
        <div>
            <canvas ref={canvas_ref} width={1030} height={765}></canvas>
            if *zoomed_in {
                <button onclick={reset_zoom}>{"Reset Zoom"}</button>
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
