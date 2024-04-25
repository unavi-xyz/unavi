use bevy::prelude::*;

use super::WiredEcsMap;

#[derive(Component)]
pub struct QueryFuture;

pub fn run_script_queries(
    world: &mut World,
    maps: &mut QueryState<&mut WiredEcsMap, Without<QueryFuture>>,
) {
    for _map in maps.iter_mut(world) {}
}
