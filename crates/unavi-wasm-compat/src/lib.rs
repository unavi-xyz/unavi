//! Cross-platform async helpers for WASM compatibility.

use std::{future::Future, time::Duration};

/// Spawn an async task.
///
/// On native: uses [`tokio::spawn`].
/// On WASM: uses [`wasm_bindgen_futures::spawn_local`].
#[cfg(not(target_family = "wasm"))]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}
#[cfg(target_family = "wasm")]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

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
    spawn(future)
}

/// Sleep for a duration.
///
/// On native: uses [`tokio::time::sleep`].
/// On WASM: uses [`gloo_timers`].
#[cfg(not(target_family = "wasm"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
#[cfg(target_family = "wasm")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
}
