#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod wasm;

#[cfg(not(target_family = "wasm"))]
pub use native::*;
#[cfg(target_family = "wasm")]
pub use wasm::*;
