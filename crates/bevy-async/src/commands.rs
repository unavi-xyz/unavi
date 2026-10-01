//! Commands queued from async tasks, applied to the world that minted them.

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
use web_time::Instant;

const CAPACITY: usize = 1024;

/// Adapts a command with any output into one that discards it, satisfying
/// [`CommandQueue::push`]'s `Out = ()` bound.
fn discard<C: Command>(command: C) -> impl Command<Out = ()> {
    move |world: &mut World| {
        command.apply(world);
    }
}

/// The sending end of the queue between this `App`'s world and its async
/// tasks.
///
/// Cloning shares the queue, so a system clones it into each task it spawns.
/// Once the `App` is dropped, sends fail rather than wait.
#[derive(Resource, Clone)]
pub struct AsyncWorld {
    tx: Sender<CommandQueue>,
}

/// The receiving end, held only by the world so that dropping the `App`
/// closes the queue.
#[derive(Resource)]
pub(crate) struct AsyncInbox(Receiver<CommandQueue>);

pub(crate) fn queue() -> (AsyncWorld, AsyncInbox) {
    let (tx, rx) = async_channel::bounded(CAPACITY);
    (AsyncWorld { tx }, AsyncInbox(rx))
}

impl AsyncWorld {
    /// A command builder that sends to this world.
    #[must_use]
    pub fn commands(&self) -> AsyncCommands {
        AsyncCommands {
            queue: CommandQueue::default(),
            tx:    self.tx.clone(),
        }
    }
}

pub(crate) fn apply(inbox: Res<AsyncInbox>, mut commands: Commands) {
    while let Ok(mut queue) = inbox.0.try_recv() {
        commands.append(&mut queue);
    }
}

/// Drains and applies queued commands directly against `world` until the queue
/// empties or `deadline` passes. Does nothing without [`crate::AsyncPlugin`].
///
/// Unlike the plugin's system, this takes effect immediately rather than at
/// the next system boundary. The deadline bounds that: a drained command may
/// wake its producer, which refills the queue before the loop looks again, so
/// an unbounded drain is a frame the producers decide the length of.
pub fn pump(world: &mut World, deadline: Instant) {
    let Some(rx) = world.get_resource::<AsyncInbox>().map(|inbox| inbox.0.clone()) else {
        return;
    };
    while let Ok(mut queue) = rx.try_recv() {
        queue.apply(world);
        if Instant::now() >= deadline {
            return;
        }
    }
}

/// A batch of commands built off the main thread, applied in order when sent.
///
/// Minted by [`AsyncWorld::commands`].
pub struct AsyncCommands {
    queue: CommandQueue,
    tx:    Sender<CommandQueue>,
}

impl AsyncCommands {
    #[must_use]
    pub fn push(mut self, command: impl Command<Out = ()>) -> Self {
        self.queue.push(command);
        self
    }

    #[must_use]
    pub fn trigger<'a, E>(mut self, event: E) -> Self
    where
        E: Event<Trigger<'a>: Default>,
    {
        self.queue
            .push(discard(bevy::ecs::system::command::trigger(event)));
        self
    }

    #[must_use]
    pub fn spawn<B>(mut self, bundle: B) -> Self
    where
        B: Bundle<Effect: NoBundleEffect>,
    {
        self.queue
            .push(discard(bevy::ecs::system::command::spawn_batch([bundle])));
        self
    }

    /// Spawns `bundle` and awaits its entity, or `None` if the world is gone.
    pub async fn send_spawn<B>(self, bundle: B) -> Option<Entity>
    where
        B: Bundle<Effect: NoBundleEffect>,
    {
        self.send_with(move |world| world.spawn(bundle).id()).await
    }

    /// Runs `f` against the world and awaits its return value, or `None` if the
    /// world is gone.
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

    /// Queues the batch, waiting while the queue is full.
    pub async fn send(self) -> Result<(), SendError<CommandQueue>> {
        self.tx.send(self.queue).await
    }

    /// Queues the batch, failing rather than waiting if the queue is full.
    pub fn try_send(self) -> Result<(), TrySendError<CommandQueue>> {
        self.tx.try_send(self.queue)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{
            AtomicBool,
            Ordering,
        },
        task::{
            Context,
            Poll,
            Waker,
        },
        time::Duration,
    };

    use super::*;

    #[derive(Component)]
    struct Marker;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(crate::AsyncPlugin);
        app
    }

    #[test]
    fn the_pump_stops_at_its_deadline() {
        static REFILLING: AtomicBool = AtomicBool::new(true);

        fn enqueue(async_world: AsyncWorld) {
            let next = async_world.clone();
            let _ = async_world
                .commands()
                .push(move |world: &mut World| {
                    world.spawn(Marker);
                    if REFILLING.load(Ordering::Relaxed) {
                        enqueue(next);
                    }
                })
                .try_send();
        }

        let mut app = app();
        enqueue(app.world().resource::<AsyncWorld>().clone());

        let world = app.world_mut();
        let started = Instant::now();
        pump(world, started + Duration::from_millis(5));
        let elapsed = started.elapsed();

        REFILLING.store(false, Ordering::Relaxed);
        pump(world, Instant::now() + Duration::from_secs(5));

        assert!(
            elapsed < Duration::from_secs(1),
            "a command that re-enqueues itself held the pump for {elapsed:?}"
        );
    }

    #[test]
    fn send_spawn_submits_queue() {
        let mut app = app();
        let async_world = app.world().resource::<AsyncWorld>().clone();

        let mut fut = Box::pin(async_world.commands().send_spawn(Marker));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(fut.as_mut().poll(&mut cx).is_pending());

        pump(app.world_mut(), Instant::now() + Duration::from_secs(5));

        let Poll::Ready(Some(ent)) = fut.as_mut().poll(&mut cx) else {
            panic!("entity not spawned");
        };
        assert!(app.world().get::<Marker>(ent).is_some());
    }

    #[test]
    fn apps_do_not_share_commands() {
        let mut first = app();
        let mut second = app();

        first
            .world()
            .resource::<AsyncWorld>()
            .commands()
            .push(|world: &mut World| {
                world.spawn(Marker);
            })
            .try_send()
            .expect("queued");

        first.update();
        second.update();

        assert_eq!(
            first
                .world_mut()
                .query::<&Marker>()
                .iter(first.world())
                .count(),
            1
        );
        assert_eq!(
            second
                .world_mut()
                .query::<&Marker>()
                .iter(second.world())
                .count(),
            0
        );
    }
}
