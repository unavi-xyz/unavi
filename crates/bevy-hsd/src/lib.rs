use std::{collections::BTreeMap, sync::Arc};

use bevy::{platform::collections::HashMap, prelude::*};
use loro::{LoroDoc, TreeID};

pub mod attributes;
mod diff;
mod subscribe;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(subscribe::subscribe_to_docs)
            .add_observer(attributes::collider::apply_collider)
            .add_observer(attributes::collider::on_collider_blobs_loaded)
            .add_observer(attributes::image::apply_image)
            .add_observer(attributes::image::on_image_blob_loaded)
            .add_observer(attributes::material::apply_material)
            .add_observer(attributes::mesh::apply_mesh)
            .add_observer(attributes::mesh::on_mesh_blobs_loaded)
            .add_observer(attributes::rigid_body::apply_rigid_body)
            .add_observer(attributes::xform::apply_xform)
            .add_systems(
                Update,
                (
                    diff::drain_diff_queues,
                    attributes::material::propagate_material_relationship,
                    attributes::material::propagate_image_to_material,
                    attributes::material::propagate_material_to_dependents,
                )
                    .chain(),
            );
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
#[require(Visibility)]
pub struct Prim(pub TreeID);

/// `TreeID` → `Entity` lookup for one HSD document. Lives on the doc entity
/// (the entity carrying `Hsd`) and is maintained by `drain_diff_queues` as
/// prims are spawned and despawned.
#[derive(Component, Default, Debug)]
pub struct HsdPrimIndex(pub HashMap<TreeID, Entity>);

/// Relationship targets declared on a prim, keyed by relationship name
/// (e.g. `"material"`, `"mesh"`). Each value is a `TreeID` of another prim
/// in the same document. Absent component = no relationships.
#[derive(Component, Default, Debug)]
pub struct HsdRelationships(pub BTreeMap<String, TreeID>);
