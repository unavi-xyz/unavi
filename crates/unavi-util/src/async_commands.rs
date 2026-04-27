use std::sync::LazyLock;

use async_channel::{Receiver, Sender};
use bevy::{ecs::world::CommandQueue, prelude::*};

const SIZE: usize = 16;

pub static ASYNC_COMMAND_QUEUE: LazyLock<(Sender<CommandQueue>, Receiver<CommandQueue>)> =
    LazyLock::new(|| async_channel::bounded(SIZE));

pub(crate) fn apply_async_commands(mut commands: Commands) {
    while let Ok(mut queue) = ASYNC_COMMAND_QUEUE.1.try_recv() {
        commands.append(&mut queue);
    }
}
