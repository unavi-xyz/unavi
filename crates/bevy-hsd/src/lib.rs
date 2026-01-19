use std::collections::BTreeSet;

use bevy::prelude::*;

use crate::attributes::Attribute;

mod attributes;

pub struct HsdPulgin;

impl Plugin for HsdPulgin {
    fn build(&self, _app: &mut App) {}
}

/// A stage is an ordered collection of layers and nodes.
/// Lazily resolves layers into a single composed state.
#[derive(Component)]
#[require(StageLayers)]
struct Stage;

#[derive(Component, Default)]
#[relationship_target(relationship = Layer)]
struct StageLayers(BTreeSet<Entity>);

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

#[derive(Component)]
enum OpinionOp<T: Attribute> {
    Set(T),
    Delete,
}
