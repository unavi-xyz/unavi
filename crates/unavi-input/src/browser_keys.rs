use bevy::prelude::*;
use wasm_bindgen::{
    JsCast,
    closure::Closure,
};
use web_sys::{
    AddEventListenerOptions,
    Event,
    HtmlElement,
    KeyboardEvent,
};

/// Keys held back from the browser because its own use of them takes the
/// app's away: `Tab` moves focus off the canvas and the rest scroll the page.
/// Held with a modifier they are the browser's — nothing here is bound with
/// one, so `Ctrl+R` and the dev tools shortcuts go through untouched.
const HELD_BACK: [&str; 6] = [
    "Tab",
    "Space",
    "ArrowUp",
    "ArrowDown",
    "ArrowLeft",
    "ArrowRight",
];

/// Takes the browser's defaults away one at a time.
///
/// Winit's own is all of them or none, and `prevent_default_event_handling`
/// is left off, so what the app binds is held back here. Focus comes with it:
/// a canvas nobody focused hears no keyboard at all.
pub fn hold_back_bound_defaults() {
    let Some(canvas) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.query_selector("canvas").ok().flatten())
        .and_then(|canvas| canvas.dyn_into::<HtmlElement>().ok())
    else {
        warn!("no canvas to hold the browser's defaults back on");
        return;
    };

    let keydown = Closure::<dyn FnMut(KeyboardEvent)>::new(|event: KeyboardEvent| {
        if event.ctrl_key() || event.meta_key() || event.alt_key() {
            return;
        }
        if HELD_BACK.contains(&event.code().as_str()) {
            event.prevent_default();
        }
    });
    let _ = canvas.add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref());
    keydown.forget();

    let focused = canvas.clone();
    let pointerdown = Closure::<dyn FnMut(Event)>::new(move |_: Event| {
        let _ = focused.focus();
    });
    let _ = canvas
        .add_event_listener_with_callback("pointerdown", pointerdown.as_ref().unchecked_ref());
    pointerdown.forget();

    for name in ["contextmenu", "wheel"] {
        let swallow = Closure::<dyn FnMut(Event)>::new(|event: Event| event.prevent_default());
        let options = AddEventListenerOptions::new();
        options.set_passive(false);
        let _ = canvas.add_event_listener_with_callback_and_add_event_listener_options(
            name,
            swallow.as_ref().unchecked_ref(),
            &options,
        );
        swallow.forget();
    }
}
