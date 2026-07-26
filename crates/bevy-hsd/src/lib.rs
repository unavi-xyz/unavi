use std::{
    collections::BTreeMap,
    sync::Arc,
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
    transform::TransformSystems,
};
use loro::{
    LoroDoc,
    TreeID,
};

pub mod attributes;
mod diff;
pub mod load;
pub mod loaded;
mod subscribe;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<load::HsdAsset>()
            .init_asset::<load::BlobAsset>()
            .register_asset_loader(load::HsdLoader)
            .register_asset_loader(load::BlobLoader)
            .add_observer(subscribe::subscribe_to_docs)
            .add_observer(attributes::collider::apply_collider)
            .add_observer(attributes::collider::on_collider_blobs_loaded)
            .add_observer(attributes::image::apply_image)
            .add_observer(attributes::image::on_image_blob_loaded)
            .add_observer(attributes::material::apply_material)
            .add_observer(attributes::mesh::apply_mesh)
            .add_observer(attributes::mesh::on_mesh_blobs_loaded)
            .add_observer(attributes::portal::apply_portal)
            .add_observer(attributes::rigid_body::apply_rigid_body)
            .add_observer(attributes::spawn::apply_spawn)
            .add_observer(attributes::xform::apply_xform)
            .add_systems(
                Update,
                (
                    commit_all_docs,
                    diff::drain_diff_queues,
                    attributes::material::propagate_material_relationship,
                    attributes::material::propagate_image_to_material,
                    attributes::material::propagate_material_to_dependents,
                    load::instance_hsd,
                    load::instance_subdocuments,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                (
                    loaded::evaluate_hsd_loaded,
                    attributes::collider::watch_collider_scale.after(TransformSystems::Propagate),
                ),
            );
    }
}

#[derive(Component)]
#[require(HsdChildren, Transform, Visibility)]
pub struct Hsd(pub Arc<LoroDoc>);

#[derive(Component, Debug, Clone, Copy)]
pub struct HsdRecordId(pub blake3::Hash);

#[derive(Component, Default)]
#[relationship_target(relationship=HsdChild, linked_spawn)]
pub struct HsdChildren(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target=HsdChildren)]
pub struct HsdChild(pub Entity);

#[derive(Component)]
#[require(Visibility, Transform)]
pub struct Prim(pub TreeID);

#[derive(Component, Default, Debug)]
pub struct HsdPrimIndex(pub HashMap<TreeID, Entity>);

fn commit_all_docs(docs: Query<&Hsd>) {
    for doc in &docs {
        doc.0.commit();
    }
}

#[derive(Component, Default, Debug)]
pub struct HsdRelationships(pub BTreeMap<String, TreeID>);
