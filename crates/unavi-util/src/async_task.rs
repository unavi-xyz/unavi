use std::{
    pin::Pin,
    sync::LazyLock,
};

use async_channel::Sender;

type Fut = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Handoff to the executor, unbounded.
///
/// A bound here is a bound on how long the frame loop may be parked;
/// backpressure belongs on what the tasks do.
pub static ASYNC_TASK: LazyLock<Sender<Fut>> = LazyLock::new(|| {
    let (tx, rx) = async_channel::unbounded();

    unavi_wasm_compat::spawn_thread(async move {
        while let Ok(fut) = rx.recv().await {
            n0_future::task::spawn(fut);
        }
    });

    tx
});

/// Spawns an async task on a global dedicated runtime.
/// On native, this is a multi-threaded tokio executor.
#[cfg(not(target_family = "wasm"))]
pub fn spawn_async_task(future: impl Future<Output = ()> + Send + 'static) {
    ASYNC_TASK
        .try_send(Box::pin(future))
        .expect("send async task");
}

#[cfg(target_family = "wasm")]
pub fn spawn_async_task(future: impl Future<Output = ()> + 'static) {
    n0_future::task::spawn(future);
}
