use std::sync::{LazyLock, Mutex};

use bevy::{ecs::world::CommandQueue, prelude::*};

pub static ASYNC_COMMAND_QUEUE: LazyLock<(
    tokio::sync::mpsc::Sender<CommandQueue>,
    Mutex<tokio::sync::mpsc::Receiver<CommandQueue>>,
)> = LazyLock::new(|| {
    let (tx, rx) = tokio::sync::mpsc::channel(16);
    (tx, Mutex::new(rx))
});

pub fn apply_async_commands(mut commands: Commands) {
    let mut guard = ASYNC_COMMAND_QUEUE.1.lock().expect("never poisons");

    while let Ok(mut queue) = guard.try_recv() {
        commands.append(&mut queue);
    }
}
