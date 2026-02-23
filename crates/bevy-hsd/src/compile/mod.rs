use bevy::prelude::*;

use crate::{
    CompiledMaterial, CompiledMesh, HsdChild, HsdMaterialEntities, HsdMeshEntities, MaterialRef,
    MeshRef,
};

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
    mesh_ents: Query<&HsdMeshEntities>,
    mat_ents: Query<&HsdMaterialEntities>,
    compiled_meshes: Query<&CompiledMesh>,
    compiled_mats: Query<&CompiledMaterial>,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
) {
    for (node_ent, hsd_child, mesh_ref, mat_ref) in &nodes {
        let Some(MeshRef(mesh_idx)) = mesh_ref else {
            continue;
        };

        let Ok(mesh_list) = mesh_ents.get(hsd_child.doc) else {
            continue;
        };

        let Some(&mesh_ent) = mesh_list.0.get(*mesh_idx) else {
            continue;
        };

        let Ok(compiled_mesh) = compiled_meshes.get(mesh_ent) else {
            continue;
        };

        let mut ent = commands.entity(node_ent);
        ent.insert(Mesh3d(compiled_mesh.0.clone()));

        // Apply compiled material or default.
        if let Some(MaterialRef(mat_idx)) = mat_ref
            && let Ok(mat_list) = mat_ents.get(hsd_child.doc)
            && let Some(&mat_ent) = mat_list.0.get(*mat_idx)
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
