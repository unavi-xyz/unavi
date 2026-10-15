//! The runtime async work runs on.

/// Runs `future` on a shared multi-threaded tokio runtime.
#[cfg(not(target_family = "wasm"))]
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    use std::sync::LazyLock;

    static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("bevy-async")
            .build()
            .expect("build tokio runtime")
    });

    RUNTIME.spawn(future);
}

/// Runs `future` on the browser's event loop.
#[cfg(target_family = "wasm")]
pub fn spawn(future: impl Future<Output = ()> + 'static) {
    n0_future::task::spawn(future);
}
