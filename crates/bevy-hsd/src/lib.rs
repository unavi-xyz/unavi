use base64::Engine;
use bevy::prelude::*;
use rand::RngCore;
use smol_str::SmolStr;

use crate::attributes::Attribute;

pub mod attributes;
mod compile;
mod load;
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
#[require(StageLayers, StageLoaded, StageCompiled)]
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
#[relationship_target(relationship = Layer, linked_spawn)]
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
#[relationship_target(relationship = Opinion, linked_spawn)]
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

/// Operation over an attribute.
#[derive(Component)]
struct OpinionOp<T: Attribute>(T);

/// Generates a random [`SmolStr`] value that is small enough to inline.
#[must_use]
pub fn random_id() -> SmolStr {
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    SmolStr::new(base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(bytes))
}
