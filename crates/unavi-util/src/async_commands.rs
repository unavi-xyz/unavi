use std::sync::{LazyLock, Mutex};

use bevy::{ecs::world::CommandQueue, prelude::*};
use tokio::sync::mpsc::{Receiver, Sender};

const SIZE: usize = 32;

pub static ASYNC_COMMAND_QUEUE: LazyLock<(Sender<CommandQueue>, Mutex<Receiver<CommandQueue>>)> =
    LazyLock::new(|| {
        let (tx, rx) = tokio::sync::mpsc::channel(SIZE);
        (tx, Mutex::new(rx))
    });

pub(crate) fn apply_async_commands(mut commands: Commands) {
    let Ok(mut guard) = ASYNC_COMMAND_QUEUE.1.try_lock() else {
        return;
    };

    while let Ok(mut queue) = guard.try_recv() {
        commands.append(&mut queue);
    }
}
