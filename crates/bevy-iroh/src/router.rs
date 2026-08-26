use bevy::prelude::*;
use iroh::protocol::{
    Router,
    RouterBuilder,
};
use unavi_util::async_task::spawn_async_task;

use crate::endpoint::IrohEndpoint;

#[derive(Component)]
pub struct IrohRouter(pub Router);

#[derive(Component)]
pub struct PendingRouter(async_channel::Receiver<Router>);

#[derive(EntityEvent)]
pub struct BuildRouter(pub Entity);

/// Stores [`RouterBuilderFn`]s, which will be consumed during the router build
/// and removed from this component after calling [`BuildRouter`].
#[derive(Component, Default)]
#[relationship_target(relationship = RouterBuilderFnTarget, linked_spawn)]
pub struct RouterBuilderFns(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = RouterBuilderFns)]
pub struct RouterBuilderFnTarget(pub Entity);

#[derive(Component)]
pub struct RouterBuilderFn(pub Option<BoxedRouterBuilder>);

pub type BoxedRouterBuilder = Box<dyn FnOnce(RouterBuilder) -> RouterBuilder + Send + Sync>;

pub(crate) fn on_build_router(
    trigger: On<BuildRouter>,
    mut commands: Commands,
    endpoints: Query<&IrohEndpoint>,
    existing: Query<(), Or<(With<IrohRouter>, With<PendingRouter>)>>,
    mut builders: Query<&mut RouterBuilderFns>,
    mut fs: Query<&mut RouterBuilderFn>,
) {
    let entity = trigger.event().event_target();

    if existing.get(entity).is_ok() {
        return;
    }

    let Ok(endpoint) = endpoints.get(entity).map(|v| v.0.clone()) else {
        warn!(%entity, "cannot build router, endpoint not found");
        return;
    };
    let Ok(fns) = builders.get_mut(entity) else {
        warn!(%entity, "cannot build router, protocols not found");
        return;
    };

    let mut collected = Vec::new();

    for fn_ent in &fns.0 {
        let mut f = fs.get_mut(*fn_ent).expect("router builder");

        commands.entity(*fn_ent).despawn();
        if let Some(f) = f.0.take() {
            collected.push(f);
        }
    }

    // A router spawns once and never rebuilds; a handler registered later is
    // silently absent.
    info!(handlers = collected.len(), "Building iroh router");

    // `Router::spawn` calls `tokio::spawn` internally, so this must run inside
    // the async runtime.
    let (tx, rx) = async_channel::bounded(1);
    spawn_async_task(async move {
        let mut builder = RouterBuilder::new(endpoint);
        for f in collected {
            builder = f(builder);
        }
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
