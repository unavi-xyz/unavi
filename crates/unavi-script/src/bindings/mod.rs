//! Lowerings of the host world onto each engine. They convert values and call
//! into [`crate::host`]; every check lives there.

#[cfg(not(target_family = "wasm"))] pub mod native;
#[cfg(target_family = "wasm")] pub mod web;
