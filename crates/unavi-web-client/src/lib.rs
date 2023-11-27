use leptos::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    unavi_app::App::new()
        .add_plugins(unavi_app::UnaviPlugin::default())
        .run();

    leptos::mount_to_body(unavi_web_common::App);
}
