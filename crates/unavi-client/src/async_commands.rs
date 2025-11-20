use std::sync::LazyLock;

use bevy::{ecs::world::CommandQueue, prelude::*};

pub static ASYNC_COMMAND_QUEUE: LazyLock<(
    flume::Sender<CommandQueue>,
    flume::Receiver<CommandQueue>,
)> = LazyLock::new(flume::unbounded);

pub fn apply_async_commands(mut commands: Commands) {
    while let Ok(mut queue) = ASYNC_COMMAND_QUEUE.1.try_recv() {
        commands.append(&mut queue);
    }
}
