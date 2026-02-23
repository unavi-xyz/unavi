use std::sync::Arc;

use bevy::prelude::*;
use loro::{Frontiers, LoroDoc};
use smol_str::SmolStr;

mod compile;
pub mod data;
mod hydrate;
pub mod hydration;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                hydrate::hydrate_hsd_docs,
                (
                    compile::material::parse_material_data,
                    compile::mesh::parse_mesh_data,
                    compile::collider::parse_collider_data,
                    compile::rigid_body::parse_rigid_body_data,
                ),
                (
                    compile::material::compile_materials,
                    compile::mesh::compile_meshes,
                ),
                compile::compile_nodes,
            )
                .chain(),
        );
    }
}

/// Source of truth. Insert on an entity to create an HSD
/// scene. All child entities are cleaned up when removed.
#[derive(Component)]
#[require(HsdVersion, HsdChildren, HsdMeshEntities, HsdMaterialEntities)]
pub struct HsdDoc(pub Arc<LoroDoc>);

/// Tracks doc version for change detection.
#[derive(Component, Default)]
struct HsdVersion(Option<Frontiers>);

/// Relationship target for all HSD-spawned entities.
#[derive(Component, Default)]
#[relationship_target(relationship = HsdChild, linked_spawn)]
pub struct HsdChildren(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = HsdChildren)]
pub struct HsdChild {
    #[relationship]
    pub doc: Entity,
}

/// Node identity.
#[derive(Component, Clone, Debug)]
pub struct NodeId(pub SmolStr);

/// Index reference from a node to a mesh resource entity.
#[derive(Component)]
pub struct MeshRef(pub usize);

/// Index reference from a node to a material resource entity.
#[derive(Component)]
pub struct MaterialRef(pub usize);

/// Compiled mesh handle stored on a mesh resource entity.
#[derive(Component)]
pub struct CompiledMesh(pub Handle<Mesh>);

/// Compiled material handle stored on a material resource entity.
#[derive(Component)]
pub struct CompiledMaterial(pub Handle<StandardMaterial>);

/// Ordered list of mesh resource entities on the doc entity.
#[derive(Component, Default)]
pub struct HsdMeshEntities(pub Vec<Entity>);

/// Ordered list of material resource entities on the doc entity.
#[derive(Component, Default)]
pub struct HsdMaterialEntities(pub Vec<Entity>);
