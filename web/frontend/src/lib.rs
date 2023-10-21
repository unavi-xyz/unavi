use leptos::*;
use unavi_web_app::App;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    unavi_engine::start();
    leptos::mount_to_body(App);
}
