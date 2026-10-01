//! Projects an [`hsd`] document into the ECS as prims, and applies the
//! attributes each prim carries.

// Bevy's `AsBindGroup` needs a higher limit.
#![recursion_limit = "256"]

use bevy::{
    asset::embedded_asset,
    prelude::*,
    transform::TransformSystems,
};

pub mod anchor;
pub mod attributes;
pub mod document;
pub mod feed;
pub mod hierarchy;
pub mod loaded;
pub mod package;
pub mod prim;
pub mod reference;
mod scene_events;

/// Applies document changes to the world. A write made before this set is
/// visible the same frame.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HsdSystems;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "attributes/shader/fallback.wgsl");

        app.add_plugins((
            MaterialPlugin::<attributes::shader::material::ShaderGraphMaterial>::default(),
            bevy_msdf::MsdfPlugin,
        ))
            .init_asset::<package::PackageAsset>()
            .init_resource::<attributes::shader::cache::ShaderGraphCache>()
            .register_asset_loader(package::PackageLoader)
            .add_observer(scene_events::resync_on_add)
            .add_observer(scene_events::resync_on_place)
            .add_observer(feed::feed_namespace)
            .add_observer(attributes::shader::cache::evict_document_shaders)
            .add_systems(
                Update,
                (
                    (
                        feed::apply_doc_deltas,
                        scene_events::discard_unplaced_events,
                        scene_events::drain_scene_events,
                        attributes::xform::apply_xform,
                        attributes::mesh::rebuild_mesh,
                        attributes::image::rebuild_image,
                        attributes::image::apply_sampler,
                        attributes::collider::rebuild_collider,
                        attributes::material::source::resolve_material_source,
                        attributes::material::pbr::rebuild_material,
                        attributes::material::pbr::propagate_image_to_material,
                        attributes::material::pbr::propagate_material_to_dependents,
                    )
                        .chain(),
                    (
                        attributes::shader::systems::rebuild_shader_material,
                        attributes::shader::systems::apply_graph_overrides,
                        package::import_packages,
                        reference::open_references,
                    )
                        .chain(),
                )
                    .chain()
                    .in_set(HsdSystems),
            )
            .configure_sets(Update, bevy_msdf::MsdfSet.after(HsdSystems))
            // `apply_xform` seeds physics positions from the anchored transform.
            .add_systems(Update, anchor::apply_anchors.before(HsdSystems))
            .add_systems(
                PostUpdate,
                loaded::evaluate_hsd_loaded.before(TransformSystems::Propagate),
            );
    }
}
