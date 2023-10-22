use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    unavi_app::start();

    leptos::mount_to_body(unavi_web_app::App);
}
