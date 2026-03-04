use bevy::prelude::*;

use crate::{CompiledMaterial, CompiledMesh, HsdChild, MaterialRef, MeshRef, cache::SceneRegistry};

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
    registries: Query<&SceneRegistry>,
    compiled_meshes: Query<&CompiledMesh>,
    compiled_mats: Query<&CompiledMaterial>,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
) {
    for (node_ent, hsd_child, mesh_ref, mat_ref) in &nodes {
        let Some(MeshRef(mesh_idx)) = mesh_ref else {
            continue;
        };

        let Ok(registry) = registries.get(hsd_child.doc) else {
            continue;
        };

        let mesh_ent = {
            let meshes = registry.0.meshes.lock().expect("meshes lock");
            meshes
                .get(*mesh_idx)
                .and_then(|inner| *inner.entity.lock().expect("entity lock"))
        };
        let Some(mesh_ent) = mesh_ent else {
            continue;
        };

        let Ok(compiled_mesh) = compiled_meshes.get(mesh_ent) else {
            continue;
        };

        let mut ent = commands.entity(node_ent);
        ent.insert(Mesh3d(compiled_mesh.0.clone()));

        // Apply compiled material or default.
        let mat_ent = mat_ref.and_then(|MaterialRef(mat_idx)| {
            let mats = registry.0.materials.lock().expect("materials lock");
            mats.get(*mat_idx)
                .and_then(|inner| *inner.entity.lock().expect("entity lock"))
        });

        if let Some(mat_ent) = mat_ent
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
