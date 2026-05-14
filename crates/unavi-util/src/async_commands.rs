use std::sync::LazyLock;

use async_channel::{Receiver, SendError, Sender, TrySendError};
use bevy::{
    ecs::{bundle::NoBundleEffect, world::CommandQueue},
    prelude::*,
};

const SIZE: usize = 1024;

pub static ASYNC_COMMAND_QUEUE: LazyLock<(Sender<CommandQueue>, Receiver<CommandQueue>)> =
    LazyLock::new(|| async_channel::bounded(SIZE));

pub(crate) fn apply_async_commands(mut commands: Commands) {
    while let Ok(mut queue) = ASYNC_COMMAND_QUEUE.1.try_recv() {
        commands.append(&mut queue);
    }
}

#[derive(Default)]
pub struct AsyncCommands {
    queue: CommandQueue,
}

impl AsyncCommands {
    #[must_use]
    pub fn push(mut self, command: impl Command) -> Self {
        self.queue.push(command);
        self
    }

    #[must_use]
    pub fn trigger<'a, E>(mut self, event: E) -> Self
    where
        E: Event<Trigger<'a>: Default>,
    {
        self.queue.push(bevy::ecs::system::command::trigger(event));
        self
    }

    #[must_use]
    pub fn spawn<B>(mut self, bundle: B) -> Self
    where
        B: Bundle<Effect: NoBundleEffect>,
    {
        self.queue
            .push(bevy::ecs::system::command::spawn_batch([bundle]));
        self
    }

    // TODO remove `send_spawn`, use `RemoteAllocator` once Bevy 0.19 releases to
    // generate an entity id outside of the world
    #[must_use]
    pub async fn send_spawn<B>(mut self, bundle: B) -> Entity
    where
        B: Bundle<Effect: NoBundleEffect>,
    {
        let (tx, rx) = async_channel::bounded(1);
        self.queue.push(move |world: &mut World| {
            let ent = world.spawn(bundle).id();
            tx.try_send(ent).expect("send");
        });
        rx.recv().await.expect("recv")
    }

    pub async fn send(self) -> Result<(), SendError<CommandQueue>> {
        ASYNC_COMMAND_QUEUE.0.send(self.queue).await
    }

    pub fn try_send(self) -> Result<(), TrySendError<CommandQueue>> {
        ASYNC_COMMAND_QUEUE.0.try_send(self.queue)?;
        Ok(())
    }
}
