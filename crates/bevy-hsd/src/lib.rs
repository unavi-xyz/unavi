use bevy::prelude::*;
use bevy::transform::TransformSystems;
use loro::LoroDoc;
use smol_str::SmolStr;
use std::sync::Arc;

pub mod cache;
pub mod data;
pub mod hydrate;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(hydrate::compile::collider::on_collider_blobs_loaded)
            .add_observer(hydrate::compile::material::handle_hsd_material_alpha_cutoff_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_alpha_mode_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_base_color_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_despawned)
            .add_observer(hydrate::compile::material::handle_hsd_material_double_sided_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_metallic_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_name_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_roughness_set)
            .add_observer(hydrate::compile::material::handle_hsd_material_spawned)
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
            .add_observer(hydrate::compile::node::on_mesh_compiled)
            .add_observer(hydrate::compile::node::on_mesh_ref_removed)
            .add_observer(hydrate::compile::node::on_mesh_ref_set);

        app.add_systems(
            FixedUpdate,
            (
                hydrate::init::init_hsd_doc,
                hydrate::sync::sync_to_hsd,
                hydrate::queue::process_hsd_queue,
                hydrate::flush::flush_script_dirty,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            hydrate::sync::sync_ecs_to_cache.after(TransformSystems::Propagate),
        );
    }
}

#[derive(Component)]
#[require(HsdChildren)]
pub struct HsdDoc(pub Arc<LoroDoc>);

/// Relationship for all HSD-spawned entities.
#[derive(Component, Default)]
#[relationship_target(relationship = HsdChild, linked_spawn)]
pub struct HsdChildren(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = HsdChildren)]
pub struct HsdChild {
    #[relationship]
    pub doc: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct NodeId(pub SmolStr);

/// Script blob hashes on node entities.
#[derive(Component, Clone, Debug)]
pub struct HsdScripts(pub Vec<blake3::Hash>);

/// WDS record ID for an HSD document.
#[derive(Component, Clone, Copy)]
pub struct HsdRecordId(pub blake3::Hash);

#[derive(Component)]
pub struct MeshRef(pub SmolStr);

#[derive(Component)]
pub struct MaterialRef(pub SmolStr);

#[derive(Component)]
pub struct CompiledMesh(pub Handle<Mesh>);

#[derive(Component)]
pub struct CompiledMaterial(pub Handle<StandardMaterial>);

/// Keeps the doc subscription alive.
#[derive(Component)]
pub struct HsdSubscription(pub loro::Subscription);
