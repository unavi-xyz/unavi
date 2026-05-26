use async_channel::Receiver;

use crate::runtime::shared::{Api, wired::input::types::InputEvent};

pub struct InputListenerRes {
    pub rx: Receiver<InputEvent>,
}

pub async fn poll(backend: &Api, listener: u32) -> anyhow::Result<Option<InputEvent>> {
    backend
        .wired_input
        .lock()
        .await
        .listeners
        .get(listener)
        .map(|r| r.rx.try_recv().ok())
        .ok_or_else(|| anyhow::anyhow!("listener not found"))
}

pub async fn drop(backend: &Api, listener: u32) -> anyhow::Result<()> {
    backend.wired_input.lock().await.listeners.remove(listener);
    Ok(())
}
