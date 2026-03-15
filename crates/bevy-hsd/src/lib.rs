use bevy::prelude::*;
use bevy::transform::TransformSystems;
use loro::LoroDoc;
use smol_str::SmolStr;
use std::sync::Arc;

pub mod cache;
pub(crate) mod compile;
pub mod data;
pub mod hydrate;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                hydrate::init::init_hsd_doc,
                hydrate::apply::flush_scene_dirty,
                hydrate::apply::apply_doc_changes,
                (
                    compile::material::parse_material_data,
                    compile::mesh::parse_mesh_data,
                    compile::collider::parse_collider_data,
                    compile::rigid_body::parse_rigid_body_data,
                ),
                (
                    compile::collider::compile_colliders,
                    compile::material::compile_materials,
                    compile::mesh::compile_meshes,
                ),
                compile::node::compile_nodes,
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
