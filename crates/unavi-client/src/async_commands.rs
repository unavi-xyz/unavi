use std::sync::{
    Arc, LazyLock, Mutex,
    mpsc::{Receiver, Sender},
};

use bevy::{ecs::world::CommandQueue, prelude::*};

pub static ASYNC_COMMAND_QUEUE: LazyLock<(
    Sender<CommandQueue>,
    Arc<Mutex<Receiver<CommandQueue>>>,
)> = LazyLock::new(|| {
    let (tx, rx) = std::sync::mpsc::channel();
    (tx, Arc::new(Mutex::new(rx)))
});

pub fn apply_async_commands(mut commands: Commands) {
    let Ok(rx) = ASYNC_COMMAND_QUEUE.1.try_lock() else {
        return;
    };
    while let Ok(mut item) = rx.try_recv() {
        commands.append(&mut item);
    }
}
