use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
};

use bevy::{
    platform::collections::HashMap,
    prelude::*,
    transform::TransformSystems,
};
use hsd::{
    id::{
        BlobId,
        DocId,
        PrimId,
    },
    state::SceneState,
};
use iroh_docs::NamespaceId;
use smol_str::SmolStr;

pub mod anchor;
pub mod attributes;
mod diff;
pub mod document;
pub mod load;
pub mod loaded;

/// Drains pending scene events and applies them to the world. Systems that
/// write to a document and want their changes reflected the same frame should
/// run before this set.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HsdCommitSet;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<load::HsdAsset>()
            .register_asset_loader(load::HsdLoader)
            .add_observer(diff::resync_on_spawn)
            .add_systems(
                Update,
                (
                    diff::drain_scene_events,
                    attributes::script::track_script,
                    attributes::prefab::track_prefab,
                    attributes::xform::apply_xform,
                    attributes::name::apply_name,
                    attributes::gravity_scale::apply_gravity_scale,
                    attributes::rigid_body::apply_rigid_body,
                    attributes::spawn::apply_spawn,
                    attributes::portal::apply_portal,
                    attributes::mesh::rebuild_mesh,
                    attributes::image::rebuild_image,
                    attributes::collider::rebuild_collider,
                    attributes::material::prepare_bound_material,
                    attributes::material::rebuild_material,
                    attributes::material::propagate_image_to_material,
                    attributes::material::propagate_material_to_dependents,
                    load::instance_hsd,
                    load::instance_prefabs,
                )
                    .chain()
                    .in_set(HsdCommitSet),
            )
            .add_observer(attributes::mesh::on_mesh_blobs_loaded)
            .add_observer(attributes::image::on_image_blob_loaded)
            .add_observer(attributes::collider::on_collider_blobs_loaded)
            .add_systems(
                PostUpdate,
                (
                    loaded::evaluate_hsd_loaded,
                    anchor::apply_anchors,
                    attributes::collider::watch_collider_scale.after(TransformSystems::Propagate),
                ),
            );
    }
}

/// A live document.
///
/// Script writes land here synchronously and reach the ECS immediately; whether
/// they reach other peers or the document's entries is decided elsewhere, by
/// the space protocol and by an explicit save.
#[derive(Component, Clone)]
#[require(HsdChildren, Transform, Visibility)]
pub struct Hsd(pub Arc<Mutex<SceneState>>);

impl Hsd {
    #[must_use]
    pub fn new(state: SceneState) -> Self {
        Self(Arc::new(Mutex::new(state)))
    }
}

/// Every document has an id from birth, so a portal receptor or a `wired:kv`
/// key never has to be remapped. A namespace-backed document's id *is* its
/// namespace; a prefab instance derives one.
#[derive(Component, Debug, Clone, Copy)]
pub struct HsdDocId(pub DocId);

/// Present only on documents that have a namespace, which is to say those that
/// can be written to storage and shared.
#[derive(Component, Debug, Clone, Copy)]
pub struct HsdNamespace(pub NamespaceId);

#[derive(Component, Default)]
#[relationship_target(relationship=HsdChild, linked_spawn)]
pub struct HsdChildren(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target=HsdChildren)]
pub struct HsdChild(pub Entity);

#[derive(Component)]
#[require(Visibility, Transform)]
pub struct Prim(pub PrimId);

#[derive(Component, Default, Debug)]
pub struct HsdPrimIndex(pub HashMap<PrimId, Entity>);

/// A prim's relationship properties: the cross-prim references, which in this
/// format share one namespace with attributes and are distinguished by a tag
/// byte rather than by name.
#[derive(Component, Default, Debug)]
pub struct HsdRelationships(pub BTreeMap<SmolStr, PrimId>);

/// A prim's bulk slots. The bytes live in the blob store; this is the hash
/// each slot's entry carries.
#[derive(Component, Default, Debug)]
pub struct HsdBulk(pub BTreeMap<SmolStr, BlobId>);

/// Pauses event draining while a batched writer (a mid-flight script fixed
/// update) holds it, so its writes reach the world atomically instead of
/// tearing across frames.
#[derive(Component, Clone)]
pub struct HsdCommitGate(pub Arc<AtomicBool>);

impl HsdCommitGate {
    #[must_use]
    pub fn is_held(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
