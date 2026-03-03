use bevy::prelude::*;

use crate::{CompiledMaterial, CompiledMesh, HsdChild, HsdEntityMap, MaterialRef, MeshRef};

pub mod collider;
pub mod material;
pub mod mesh;
pub mod rigid_body;

/// Assign compiled Mesh3d/MeshMaterial3d to nodes that
/// reference compiled resource entities.
pub fn compile_nodes(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    nodes: Query<(Entity, &HsdChild, Option<&MeshRef>, Option<&MaterialRef>), Without<Mesh3d>>,
    entity_maps: Query<&HsdEntityMap>,
    compiled_meshes: Query<&CompiledMesh>,
    compiled_mats: Query<&CompiledMaterial>,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
) {
    for (node_ent, hsd_child, mesh_ref, mat_ref) in &nodes {
        let Some(MeshRef(mesh_idx)) = mesh_ref else {
            continue;
        };

        let Ok(map) = entity_maps.get(hsd_child.doc) else {
            continue;
        };

        let Some(&Some(mesh_ent)) = map.meshes.get(*mesh_idx) else {
            continue;
        };

        let Ok(compiled_mesh) = compiled_meshes.get(mesh_ent) else {
            continue;
        };

        let mut ent = commands.entity(node_ent);
        ent.insert(Mesh3d(compiled_mesh.0.clone()));

        // Apply compiled material or default.
        if let Some(MaterialRef(mat_idx)) = mat_ref
            && let Some(&Some(mat_ent)) = map.materials.get(*mat_idx)
            && let Ok(compiled_mat) = compiled_mats.get(mat_ent)
        {
            ent.insert(MeshMaterial3d(compiled_mat.0.clone()));
            continue;
        }

        let mat = default_material
            .get_or_insert_with(|| asset_server.add(StandardMaterial::default()))
            .clone();
        ent.insert(MeshMaterial3d(mat));
    }
}
