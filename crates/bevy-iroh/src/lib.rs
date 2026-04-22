use bevy::prelude::*;
use iroh::{
    Endpoint,
    protocol::{Router, RouterBuilder},
};

mod endpoint;
mod router;

pub struct IrohPlugin;

impl Plugin for IrohPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(endpoint::on_load_endpoint)
            .add_observer(router::on_build_router)
            .add_systems(FixedUpdate, endpoint::recieve_endpoint);
    }
}

#[derive(Component)]
#[require(RouterBuilderFns)]
pub struct IrohEndpoint(pub Endpoint);

#[derive(Event, Clone)]
pub struct LoadEndpoint {
    pub discovery_mdns: bool,
}

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
