use bevy::prelude::*;

use crate::attributes::Attribute;

mod attributes;
mod compile;

pub struct HsdPulgin;

impl Plugin for HsdPulgin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, compile::compile_stages);
    }
}

/// A stage is an ordered collection of layers and nodes.
/// Lazily resolves layers into a single composed state.
#[derive(Component)]
#[require(StageLayers, Compiled)]
struct Stage;

/// Lazy compilation flag.
/// Set to false to re-compile the stage.
#[derive(Component, Default)]
struct Compiled(bool);

#[derive(Component, Default)]
#[relationship_target(relationship = Layer)]
struct StageLayers(Vec<Entity>);

/// A layer is a collection of opinions.
#[derive(Component)]
#[relationship(relationship_target = StageLayers)]
#[require(LayerStrength, LayerOpinions)]
struct Layer {
    #[relationship]
    stage: Entity,
}

#[derive(Component, Default)]
struct LayerStrength(usize);

#[derive(Component, Default)]
#[relationship_target(relationship = Opinion)]
struct LayerOpinions(Vec<Entity>);

/// An opinion is an operation over some entity's attribute.
#[derive(Component)]
#[relationship(relationship_target = LayerOpinions)]
struct Opinion {
    #[relationship]
    layer: Entity,
}

#[derive(Component)]
struct OpinionTarget(Entity);

#[derive(Component, Clone)]
enum OpinionOp<T: Clone> {
    Set(T),
    Delete,
}

impl<T: Attribute + Clone> OpinionOp<T> {
    fn merge(self, next: &Self) -> Self {
        match (self, next) {
            (Self::Delete, _) | (_, Self::Delete) => Self::Delete,
            (Self::Set(s), Self::Set(n)) => Self::Set(s.merge(n)),
        }
    }
}
