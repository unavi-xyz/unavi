//! A real local [`Store`] for tests that need documents.

use std::future::Future;

use iroh::{
    Endpoint,
    SecretKey,
    endpoint::presets::N0DisableRelay,
};
use iroh_docs::Author;
use unavi_store::{
    Store,
    StoreBuilder,
};

/// Runs `future` on the task runtime [`bevy_async::task::spawn`] uses, and
/// blocks this thread for the result.
pub fn block_on<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) -> T {
    let (tx, rx) = async_channel::bounded(1);
    bevy_async::task::spawn(async move {
        tx.send(future.await).await.expect("send result");
    });
    rx.recv_blocking().expect("task result")
}

pub fn store() -> Store {
    block_on(async {
        let secret_key = SecretKey::generate();
        let author = Author::from_bytes(&secret_key.to_bytes());
        let endpoint = Endpoint::builder(N0DisableRelay)
            .secret_key(secret_key)
            .bind()
            .await
            .expect("bind endpoint");
        StoreBuilder::new(endpoint, author)
            .build()
            .await
            .expect("build store")
    })
}
