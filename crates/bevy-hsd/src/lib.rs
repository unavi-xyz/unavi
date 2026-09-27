// Bevy's `AsBindGroup` needs a higher limit.
#![recursion_limit = "256"]

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        Mutex,
    },
};

use bevy::{
    asset::embedded_asset,
    platform::collections::HashMap,
    prelude::*,
    transform::TransformSystems,
};
use hsd::{
    id::{
        DocId,
        PrimId,
    },
    property::name::PropName,
    state::HsdState,
};
use wds::document::Document;

pub mod anchor;
pub mod attributes;
mod drain;
pub mod feed;
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
        embedded_asset!(app, "attributes/shader/fallback.wgsl");

        app.add_plugins((
            MaterialPlugin::<attributes::shader::ShaderGraphMaterial>::default(),
            bevy_msdf::MsdfPlugin,
        ))
            .init_asset::<load::HsdAsset>()
            .init_resource::<attributes::shader::ShaderGraphCache>()
            .register_asset_loader(load::HsdLoader)
            .add_observer(drain::resync_on_spawn)
            .add_observer(feed::feed_namespace)
            .add_observer(attributes::shader::evict_document_shaders)
            .add_systems(
                Update,
                (
                    // Split in two: Bevy's tuple `IntoSystemConfigs` impl has a fixed arity.
                    (
                        // Before the drain, so what the store changed reaches
                        // the world the same frame.
                        feed::apply_doc_deltas,
                        drain::discard_held_events,
                        drain::drain_scene_events,
                        attributes::xform::apply_xform,
                        attributes::mesh::rebuild_mesh,
                        attributes::image::rebuild_image,
                        attributes::image::apply_sampler,
                        attributes::collider::rebuild_collider,
                        attributes::material_source::resolve_material_source,
                        attributes::material::rebuild_material,
                        attributes::material::propagate_image_to_material,
                        attributes::material::propagate_material_to_dependents,
                    )
                        .chain(),
                    (
                        attributes::shader::rebuild_shader_material,
                        attributes::shader::apply_graph_overrides,
                        load::instance_hsd,
                        load::realize_refs,
                    )
                        .chain(),
                )
                    .chain()
                    .in_set(HsdCommitSet),
            )
            // The renderer must run after `HsdCommitSet`, or a label's text is a frame stale.
            .configure_sets(Update, bevy_msdf::MsdfSet.after(HsdCommitSet))
            // `apply_xform` seeds a body's physics position from the document's
            // transform, so the anchor must place the document first.
            .add_systems(Update, anchor::apply_anchors.before(HsdCommitSet))
            .add_systems(
                PostUpdate,
                loaded::evaluate_hsd_loaded.before(TransformSystems::Propagate),
            );
    }
}

/// A live document.
///
/// Script writes land here synchronously and reach the ECS immediately;
/// whether they reach peers or the document's entries is decided by the space
/// protocol and by an explicit save.
#[derive(Component, Clone)]
#[require(HsdChildren, Transform, Visibility)]
pub struct Hsd(pub Arc<Mutex<HsdState>>);

impl Hsd {
    #[must_use]
    pub fn new(state: HsdState) -> Self {
        Self(Arc::new(Mutex::new(state)))
    }
}

/// A document that is not in the scene.
///
/// A script writes to it exactly as to a live one; nothing of it is drawn,
/// simulated or reachable by position. [`anchor::place`] inserts it whole, so
/// a just-built document appears where it was put rather than arriving at the
/// origin and moving.
#[derive(Component, Clone)]
pub struct HsdHeld(pub Arc<Mutex<HsdState>>);

/// A namespace-backed document's id is its namespace; a reference site
/// derives one.
#[derive(Component, Debug, Clone, Copy)]
pub struct HsdDocId(pub DocId);

/// The document a realized reference stands for.
///
/// Its [`HsdDocId`] is the reference *site*, since two prims may name one
/// document; this is what says which document that is.
#[derive(Component, Debug, Clone, Copy)]
pub struct HsdSource(pub DocId);

/// Present only on namespace-backed documents, which can be written to storage
/// and shared.
///
/// Holds the document open rather than naming it, so retention cannot evict
/// one the scene is still drawing.
#[derive(Component, Debug, Clone)]
pub struct HsdNamespace(pub Document);

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
/// format share one group with attributes and are distinguished by a tag
/// byte rather than by name.
#[derive(Component, Default, Debug)]
pub struct HsdRelationships(pub BTreeMap<PropName, PrimId>);
