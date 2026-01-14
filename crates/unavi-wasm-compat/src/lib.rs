//! Cross-platform async helpers for WASM compatibility.

use std::{future::Future, time::Duration};

/// Spawn an async thread.
///
/// On native: uses [`std::thread::spawn`].
/// On WASM: uses [`wasm_bindgen_futures::spawn_local`].
///
/// # Panics
///
/// Panics if a tokio runtime could not be intialized in the new thread.
#[cfg(not(target_family = "wasm"))]
pub fn spawn_thread<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");
        rt.block_on(future);
    });
}
#[cfg(target_family = "wasm")]
pub fn spawn_thread<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    n0_future::task::spawn(future);
}

/// Sleep a thread for a duration.
///
/// On native: uses [`std::thread::sleep`].
/// On WASM:
#[cfg(not(target_family = "wasm"))]
pub fn sleep_thread(duration: Duration) {
    std::thread::sleep(duration);
}
#[cfg(target_family = "wasm")]
pub fn sleep_thread(duration: Duration) {
    // n0_future::time::sleep(duration);
}
