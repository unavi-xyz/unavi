use std::sync::LazyLock;

use async_channel::{Receiver, Sender, TrySendError};
use bevy::{ecs::world::CommandQueue, prelude::*};

const SIZE: usize = 256;

pub static ASYNC_COMMAND_QUEUE: LazyLock<(Sender<CommandQueue>, Receiver<CommandQueue>)> =
    LazyLock::new(|| async_channel::bounded(SIZE));

pub fn try_send_command(command: impl Command) -> Result<(), TrySendError<CommandQueue>> {
    let mut q = CommandQueue::default();
    q.push(command);
    ASYNC_COMMAND_QUEUE.0.try_send(q)?;
    Ok(())
}

pub(crate) fn apply_async_commands(mut commands: Commands) {
    while let Ok(mut queue) = ASYNC_COMMAND_QUEUE.1.try_recv() {
        commands.append(&mut queue);
    }
}
