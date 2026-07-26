use std::sync::LazyLock;

use async_channel::{
    Receiver,
    SendError,
    Sender,
    TrySendError,
};
use bevy::{
    ecs::{
        bundle::NoBundleEffect,
        world::CommandQueue,
    },
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
        self.send().await.expect("async command queue closed");
        rx.recv().await.expect("recv")
    }

    /// Runs `f` against the world and awaits its return value, or `None` if the
    /// command queue is closed.
    pub async fn send_with<T, F>(mut self, f: F) -> Option<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut World) -> T + Send + 'static,
    {
        let (tx, rx) = async_channel::bounded(1);
        self.queue.push(move |world: &mut World| {
            let _ = tx.try_send(f(world));
        });
        self.send().await.ok()?;
        rx.recv().await.ok()
    }

    pub async fn send(self) -> Result<(), SendError<CommandQueue>> {
        ASYNC_COMMAND_QUEUE.0.send(self.queue).await
    }

    pub fn try_send(self) -> Result<(), TrySendError<CommandQueue>> {
        ASYNC_COMMAND_QUEUE.0.try_send(self.queue)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::task::{
        Context,
        Poll,
        Waker,
    };

    use super::*;

    #[derive(Component)]
    struct Marker;

    #[test]
    fn send_spawn_submits_queue() {
        let mut fut = Box::pin(AsyncCommands::default().send_spawn(Marker));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(fut.as_mut().poll(&mut cx).is_pending());

        let mut world = World::new();
        while let Ok(mut queue) = ASYNC_COMMAND_QUEUE.1.try_recv() {
            queue.apply(&mut world);
        }

        let Poll::Ready(ent) = fut.as_mut().poll(&mut cx) else {
            panic!("entity not spawned");
        };
        assert!(world.get::<Marker>(ent).is_some());
    }
}
