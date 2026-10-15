//! The endpoint's router, spawned once with every protocol registered before
//! [`BuildRouter`].

use bevy::prelude::*;
use bevy_async::task;
use iroh::protocol::{
    Router,
    RouterBuilder,
};

use crate::endpoint::IrohEndpoint;

#[derive(Component)]
pub struct IrohRouter(pub Router);

#[derive(Component)]
pub struct PendingRouter(async_channel::Receiver<Router>);

/// Spawns the router of the targeted endpoint, consuming its
/// [`RouterProtocols`]. A router never rebuilds.
#[derive(EntityEvent)]
pub struct BuildRouter(pub Entity);

/// Installs a protocol on a router builder.
pub type Protocol = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;

/// Protocols waiting for the endpoint's router to build.
#[derive(Component, Default)]
pub struct RouterProtocols(Vec<Protocol>);

/// Queues `protocol` on the endpoint entity this command targets. One queued
/// after the router built is never installed, and is logged.
pub fn accept(
    protocol: impl FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync + 'static,
) -> impl EntityCommand {
    move |mut entity: EntityWorldMut| {
        if entity.contains::<IrohRouter>() || entity.contains::<PendingRouter>() {
            warn!(entity = %entity.id(), "router already built; a late protocol is dropped");
            return;
        }
        entity
            .entry::<RouterProtocols>()
            .or_default()
            .get_mut()
            .0
            .push(Box::new(protocol));
    }
}

pub(crate) fn on_build_router(
    trigger: On<BuildRouter>,
    mut commands: Commands,
    mut endpoints: Query<
        (&IrohEndpoint, &mut RouterProtocols),
        (Without<IrohRouter>, Without<PendingRouter>),
    >,
) {
    let entity = trigger.event().event_target();

    let Ok((endpoint, mut protocols)) = endpoints.get_mut(entity) else {
        return;
    };
    let endpoint = endpoint.0.clone();
    let protocols = std::mem::take(&mut protocols.0);

    info!(protocols = protocols.len(), "Building iroh router");

    // `Router::spawn` calls `tokio::spawn` internally, so this must run inside
    // the async runtime.
    let (tx, rx) = async_channel::bounded(1);
    task::spawn(async move {
        let builder = protocols
            .into_iter()
            .fold(RouterBuilder::new(endpoint), |builder, protocol| {
                protocol(builder)
            });
        tx.send(builder.spawn()).await.ok();
    });

    commands.entity(entity).insert(PendingRouter(rx));
}

pub(crate) fn receive_router(loading: Query<(Entity, &PendingRouter)>, mut commands: Commands) {
    for (entity, pending) in &loading {
        let Ok(router) = pending.0.try_recv() else {
            continue;
        };
        commands
            .entity(entity)
            .insert(IrohRouter(router))
            .remove::<PendingRouter>();
    }
}
