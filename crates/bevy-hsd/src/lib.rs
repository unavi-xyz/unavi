use std::sync::Arc;

use bevy::prelude::*;
use loro::{LoroDoc, TreeID};

mod attributes;
mod diff;
mod subscribe;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(subscribe::subscribe_to_docs)
            .add_observer(attributes::name::apply_name)
            .add_observer(attributes::xform::apply_xform)
            .add_systems(Update, diff::drain_diff_queues);
    }
}

#[derive(Component)]
#[require(HsdChildren)]
pub struct Hsd(pub Arc<LoroDoc>);

#[derive(Component, Default)]
#[relationship_target(relationship=HsdChild)]
pub struct HsdChildren(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target=HsdChildren)]
pub struct HsdChild(pub Entity);

#[derive(Component)]
pub struct Prim(pub TreeID);
