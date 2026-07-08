use crate::app::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{console, window, CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;

mod app;
mod geometry;

fn switch(routes: AppView) -> Html {
    html! { <App {routes} /> }
}

#[component(Main)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<AppView> render={switch} />
        </BrowserRouter>
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub routes: AppView,
}

#[component]
fn App(props: &Props) -> Html {
    let routes = props.routes;
    let navigator = use_navigator().unwrap();
    let canvas_ref = use_node_ref();

    // let selecting = use_mut_ref(|| false);
    let selection = use_mut_ref::<Option<(i32, i32, i32, i32)>, _>(|| None);
    let zoomed_in = use_state(|| false);
    let partial_culling = use_state(|| !matches!(routes, AppView::NoClip));
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
        let navigator = navigator.clone();
        use_effect_with(canvas_ref, move |_canvas_ref| {
            let pointerup = Closure::<dyn Fn(Event)>::new(move |_e: Event| {
                let mut selection = selection.borrow_mut();
                match &*selection {
                    Some(s) if s.2 == i32::MAX || s.3 == i32::MAX => {
                        // insufficient drag distance; this is just a click
                        let app = app.borrow();
                        console::log_1(&format!("Clicked: {:?}", app.selected_faces).into());
                        let faces: Vec<_> = app.selected_faces.iter().take(2).collect();
                        match app.view {
                            AppView::Painter { .. } | AppView::SliceView { .. } => {
                                match faces.len() {
                                    1 => navigator.push(&AppView::SliceView { face: SliceThing::OneFace(*faces[0]), idx: 1 }),
                                    2 => navigator.push(&AppView::SliceView { face: SliceThing::TwoFace(*faces[1], *faces[0]), idx: 1 }),
                                    _ => ()
                                }
                            },
                            AppView::Main | AppView::NoClip => {
                                console::log_1(&"pushing painter view".into());
                                if !faces.is_empty() {
                                    navigator.push(&AppView::Painter { face: *faces.into_iter().last().unwrap() });
                                }
                            }
                            _ => (),
                        };
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
    {
        let app = app.clone();
        use_effect_with(routes, move |routes| {
            let mut app = app.borrow_mut();
            app.view = *routes;
            app.restart();
            app.render();
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

    let click_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.back();
        })
    };

    let onchange_partial_culling = {
        let partial_culling = partial_culling.clone();
        let navigator = navigator.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            partial_culling.set(input.checked());
            if input.checked() {
                navigator.replace(&AppView::Main);
            } else {
                navigator.replace(&AppView::NoClip);
            }
        })
    };

    let prev_cut = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            if let AppView::SliceView { face, idx } = routes {
                if idx > 1 {
                    navigator.push(&AppView::SliceView { face, idx: idx - 1 })
                }
            }
        })
    };

    let next_cut = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            if let AppView::SliceView { face, idx } = routes {
                navigator.push(&AppView::SliceView { face, idx: idx + 1 })
            }
        })
    };

    html! {
        <div>
            <canvas ref={canvas_ref} width={1030} height={765}></canvas>
            <div class="panel">
            <button onclick={click_back}>{"Back"}</button>
            if *zoomed_in {
                <button onclick={reset_zoom}>{"Reset Zoom"}</button>
            }
            {
            match routes{
                AppView::SliceView { .. } => {
                    html! {
                        <div>
                            <button onclick={prev_cut}>{"Prev Cut"}</button>
                            <button onclick={next_cut}>{"Next Cut"}</button>
                        </div>
                    }
                },
                AppView::Main | AppView::NoClip => {
                html! {
                    <label>
                    <input
                        type="checkbox"
                        checked={*partial_culling}
                        onchange={onchange_partial_culling} />
                    {"Partial Culling"}
                    </label>
                }
                }
                _ => html! {}
            }
            }
                </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<Main>::new().render();
}
