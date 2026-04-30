#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

pub mod shared;

#[cfg(not(target_family = "wasm"))]
pub mod native;
#[cfg(target_family = "wasm")]
pub mod web;

#[cfg_attr(target_family = "wasm", wasm_bindgen(getter_with_clone))]
#[cfg_attr(target_family = "wasm", derive(Clone))]
pub struct Runtime {
    pub backend: shared::RuntimeBackend,
    #[cfg(not(target_family = "wasm"))]
    pub native: native::NativeRuntime,
    #[cfg(target_family = "wasm")]
    pub web: web::WebRuntime,
}
