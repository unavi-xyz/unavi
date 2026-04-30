#[cfg(not(target_family = "wasm"))]
pub mod native;
#[cfg(target_family = "wasm")]
pub mod web;

pub struct Runtime {
    #[cfg(not(target_family = "wasm"))]
    pub native: native::NativeRuntime,
}
