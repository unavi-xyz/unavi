use bevy::prelude::*;

#[cfg(not(target_family = "wasm"))]
mod native;
#[cfg(target_family = "wasm")]
mod web;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
#[relationship(relationship_target = Scripts)]
pub struct Engine(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = Engine)]
pub struct Scripts(Vec<Entity>);
