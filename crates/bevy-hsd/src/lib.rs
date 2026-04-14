//! Bevy plugin for rendering HSD (Hyperspace Document) scenes.
//!
//! HSD is a Loro CRDT-backed scene format. Documents arrive as content-addressed
//! blobs, are subscribed to for incremental diffs, and are compiled into standard
//! Bevy assets (meshes, materials, images) via an event-driven observer pipeline.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use avian3d::schedule::PhysicsSystems;
use bevy::prelude::*;
use bevy::transform::TransformSystems;
use loro::LoroDoc;
use smol_str::SmolStr;

use cache::SceneRegistryInner;

pub mod cache;
pub mod data;
pub mod hydrate;
pub mod load_hsd;

pub use load_hsd::LoadHsdFile;

pub struct HsdPlugin;

/// Global map from document record ID to its Bevy entity and scene registry.
///
/// Populated by `register_doc_registries` after `init_hsd_doc` runs. Used by
/// event handlers to look up the registry from a hash instead of querying by
/// entity.
#[derive(Resource, Default)]
pub struct DocRegistryMap(pub HashMap<blake3::Hash, (Entity, Arc<SceneRegistryInner>)>);

/// Named blob-id assets from the HSD document's `assets` map.
///
/// Maps asset name to the blake3 hash of the referenced blob.
#[derive(Component, Clone, Debug, Default)]
pub struct HsdAssets(pub BTreeMap<SmolStr, blake3::Hash>);

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DocRegistryMap>();
        app.init_resource::<load_hsd::PendingHsdLoads>();
        app.add_observer(load_hsd::on_load_hsd_file);

        app.add_observer(hydrate::compile::collider::on_collider_blobs_loaded)
            .add_observer(hydrate::compile::image::handle_hsd_image_despawned)
            .add_observer(hydrate::compile::image::handle_hsd_image_spawned)
            .add_observer(hydrate::compile::image::on_image_blobs_loaded)
            .add_observer(hydrate::compile::image::on_image_compiled)
            .add_observer(hydrate::compile::material::handle_hsd_material_alpha_cutoff_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_alpha_mode_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_base_color_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_base_color_texture_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_despawned)
            .add_observer(hydrate::compile::material::handle_hsd_material_double_sided_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_emissive_texture_set)
            .add_observer(
                hydrate::compile::material::handle_hsd_material_metallic_roughness_texture_set,
            )
            .add_observer(hydrate::compile::material::handle_hsd_material_metallic_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_name_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_normal_texture_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_occlusion_texture_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_roughness_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_spawned)
            .add_observer(hydrate::compile::material::handle_hsd_material_unlit_set)
            .add_observer(hydrate::compile::material::on_material_blobs_loaded)
            .add_observer(hydrate::compile::mesh::handle_hsd_mesh_despawned)
            .add_observer(hydrate::compile::mesh::handle_hsd_mesh_geometry_set)
            .add_observer(hydrate::compile::mesh::handle_hsd_mesh_spawned)
            .add_observer(hydrate::compile::mesh::on_mesh_blobs_loaded)
            .add_observer(hydrate::compile::node::handle_hsd_node_collider_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_despawned)
            .add_observer(hydrate::compile::node::handle_hsd_node_material_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_mesh_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_name_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_parent_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_rigid_body_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_scripts_set)
            .add_observer(hydrate::compile::node::handle_hsd_node_spawned)
            .add_observer(hydrate::compile::node::handle_hsd_node_transform_set)
            .add_observer(hydrate::compile::node::on_material_compiled)
            .add_observer(hydrate::compile::node::on_material_ref_set)
            .add_observer(hydrate::compile::node::on_mesh_compiled)
            .add_observer(hydrate::compile::node::on_mesh_ref_removed)
            .add_observer(hydrate::compile::node::on_mesh_ref_set);

        app.add_systems(
            FixedPostUpdate,
            (hydrate::compile::node::guard_physics_scale, ApplyDeferred)
                .chain()
                .before(PhysicsSystems::StepSimulation),
        )
        .add_systems(
            FixedUpdate,
            (
                load_hsd::start_hsd_loads,
                load_hsd::poll_hsd_file_loads,
                hydrate::init::init_hsd_doc,
                hydrate::init::register_doc_registries,
                hydrate::sync::sync_to_hsd,
                hydrate::queue::process_hsd_queue,
                bevy_wds::blob_deps::load_blob_deps,
                hydrate::compile::material::recompile_changed_materials,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            hydrate::sync::sync_ecs_to_cache.after(TransformSystems::Propagate),
        );
    }
}

/// Root Loro CRDT document for one HSD scene.
#[derive(Component)]
#[require(HsdChildren)]
pub struct HsdDoc(pub Arc<LoroDoc>);

/// All ECS entities spawned from an HSD document are children of the doc
/// entity, so the whole scene can be cleanly despawned in one operation.
#[derive(Component, Default)]
#[relationship_target(relationship = HsdChild, linked_spawn)]
pub struct HsdChildren(Vec<Entity>);

/// Marks an entity as owned by the given HSD doc entity.
#[derive(Component)]
#[relationship(relationship_target = HsdChildren)]
pub struct HsdChild {
    #[relationship]
    pub doc: Entity,
}

pub use hydrate::compile::image::{CompiledImage, ImageId};
pub use hydrate::compile::material::{CompiledMaterial, MaterialParams};
pub use hydrate::compile::mesh::CompiledMesh;
pub use hydrate::compile::node::{MaterialRef, MeshRef};
pub use load_hsd::HsdFilePath;

/// Stable HSD tree ID kept on the entity for cross-system lookup.
#[derive(Component, Clone, Debug)]
pub struct NodeId(pub SmolStr);

/// Blob hashes of WASM scripts declared on this node.
#[derive(Component, Clone, Debug)]
pub struct HsdScripts(pub Vec<blake3::Hash>);

/// Cached physics spec so colliders can be restored after scale-suppression.
///
/// See `guard_physics_scale` in `compile::node`.
#[derive(Component, Clone, Debug, Default)]
pub struct HsdNodePhysics {
    pub collider: Option<data::HsdCollider>,
    pub rigid_body: Option<data::HsdRigidBody>,
}

/// WDS content-address of the document blob.
#[derive(Component, Clone, Copy)]
pub struct HsdRecordId(pub blake3::Hash);

/// Keeps the Loro subscription alive — dropping this stops diff delivery.
#[derive(Component)]
pub struct HsdSubscription(pub loro::Subscription);
