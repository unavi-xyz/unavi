//! Cross-platform spawn helper.
//!
//! Uses `wasm_bindgen_futures::spawn_local` on WASM (single-threaded),
//! and `tokio::spawn` on native platforms.

use std::future::Future;

#[cfg(target_family = "wasm")]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_family = "wasm"))]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}
