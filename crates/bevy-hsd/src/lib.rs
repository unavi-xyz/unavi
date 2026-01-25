use bevy::prelude::*;
use loro::LoroMapValue;

pub mod attributes;
mod compile;
mod load;
mod merge;
pub mod stage;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (load::load_stages, compile::compile_stages).chain(),
        );
    }
}

/// A stage is an ordered collection of layers and nodes.
/// Lazily compiles layers into a single composed state.
#[derive(Component)]
#[require(StageLoaded, StageCompiled, StageLayers, StageNodes)]
pub struct Stage(pub stage::StageData);

/// Lazy loading flag.
/// Set to false to re-load stage data into the ECS.
#[derive(Component, Default)]
struct StageLoaded(bool);

/// Lazy compilation flag.
/// Set to false to re-compile the stage scene from ECS data.
#[derive(Component, Default)]
struct StageCompiled(bool);

#[derive(Component, Default)]
#[relationship_target(relationship = StageNode, linked_spawn)]
struct StageNodes(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = StageNodes)]
struct StageNode {
    #[relationship]
    stage: Entity,
}

#[derive(Component, Default)]
#[relationship_target(relationship = Layer, linked_spawn)]
struct StageLayers(Vec<Entity>);

/// A layer is a collection of opinions.
#[derive(Component)]
#[relationship(relationship_target = StageLayers)]
#[require(LayerEnabled, LayerStrength, LayerOpinions)]
struct Layer {
    #[relationship]
    stage: Entity,
}

#[derive(Component)]
struct LayerEnabled(bool);

impl Default for LayerEnabled {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component, Default)]
struct LayerStrength(usize);

#[derive(Component, Default)]
#[relationship_target(relationship = Opinion, linked_spawn)]
struct LayerOpinions(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = LayerOpinions)]
struct Opinion {
    #[relationship]
    layer: Entity,
}

#[derive(Component)]
struct OpinionTarget(Entity);

#[derive(Component)]
struct OpinionAttrs(LoroMapValue);
