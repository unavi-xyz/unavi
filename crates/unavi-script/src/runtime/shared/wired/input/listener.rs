use async_channel::Receiver;

use crate::runtime::shared::{RuntimeBackend, wired::input::types::InputEvent};

pub struct InputListenerRes {
    pub rx: Receiver<InputEvent>,
}

pub fn poll(backend: &RuntimeBackend, listener: u32) -> anyhow::Result<Option<InputEvent>> {
    backend
        .wired_input
        .try_lock()?
        .listeners
        .get(listener)
        .map(|r| r.rx.try_recv().ok())
        .ok_or_else(|| anyhow::anyhow!("listener not found"))
}

pub fn drop(backend: &RuntimeBackend, listener: u32) -> anyhow::Result<()> {
    backend.wired_input.try_lock()?.listeners.remove(listener);
    Ok(())
}
