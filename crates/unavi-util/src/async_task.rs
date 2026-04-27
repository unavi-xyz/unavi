use std::{pin::Pin, sync::LazyLock};

use async_channel::Sender;

type Fut = Pin<Box<dyn Future<Output = ()> + Send>>;

const SIZE: usize = 8;

pub static ASYNC_TASK: LazyLock<Sender<Fut>> = LazyLock::new(|| {
    let (tx, rx) = async_channel::bounded(SIZE);

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
        .send_blocking(Box::pin(future))
        .expect("send async task");
}

#[cfg(target_family = "wasm")]
pub fn spawn_async_task(future: impl Future<Output = ()> + 'static) {
    n0_future::task::spawn(future);
}
