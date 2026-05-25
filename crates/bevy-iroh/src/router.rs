use bevy::prelude::*;
use iroh::protocol::{
    Router,
    RouterBuilder,
};

use crate::endpoint::IrohEndpoint;

#[derive(Component)]
pub struct IrohRouter(pub Router);

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
    mut builders: Query<&mut RouterBuilderFns>,
    mut fs: Query<&mut RouterBuilderFn>,
) {
    let entity = trigger.event().event_target();

    let Ok(endpoint) = endpoints.get(entity).map(|v| v.0.clone()) else {
        warn!(%entity, "cannot build router, endpoint not found");
        return;
    };
    let Ok(fns) = builders.get_mut(entity) else {
        warn!(%entity, "cannot build router, protocols not found");
        return;
    };

    let mut builder = RouterBuilder::new(endpoint);

    for fn_ent in &fns.0 {
        let mut f = fs.get_mut(*fn_ent).expect("router builder");

        commands.entity(*fn_ent).despawn();
        let Some(f) = f.0.take() else {
            continue;
        };

        builder = f(builder);
    }

    let router = builder.spawn();

    commands.entity(entity).insert(IrohRouter(router));
}
