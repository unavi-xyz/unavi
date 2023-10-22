use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    unavi_app::start();
    leptos::mount_to_body(unavi_web_app::App);
}
