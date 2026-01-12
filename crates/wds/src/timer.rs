//! Cross-platform timer helper.
//!
//! Uses `gloo_timers` on WASM, and `tokio::time` on native platforms.

use std::time::Duration;

#[cfg(target_family = "wasm")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
}

#[cfg(not(target_family = "wasm"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}
