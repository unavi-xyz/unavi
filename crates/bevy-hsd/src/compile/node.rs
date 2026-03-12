use bevy::prelude::*;

use crate::{
    CompiledMaterial, CompiledMesh, HsdChild, MaterialRef, MeshRef, NodeId, cache::SceneRegistry,
    compile::Uncompiled,
};

/// Assign compiled Mesh3d/MeshMaterial3d to nodes that
/// reference compiled resource entities.
pub fn compile_nodes(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    nodes: Query<
        (
            Entity,
            &HsdChild,
            Option<&MeshRef>,
            Option<&MaterialRef>,
            Option<&Mesh3d>,
        ),
        (
            With<NodeId>,
            Or<(Changed<MeshRef>, Changed<MaterialRef>, With<Uncompiled>)>,
        ),
    >,
    registries: Query<&SceneRegistry>,
    compiled_meshes: Query<&CompiledMesh>,
    compiled_mats: Query<&CompiledMaterial>,
    mut default_material: Local<Option<Handle<StandardMaterial>>>,
) {
    for (node_ent, hsd_child, mesh_ref, mat_ref, cur_mesh) in &nodes {
        commands.entity(node_ent).remove::<Uncompiled>();

        let Some(MeshRef(mesh_id)) = mesh_ref else {
            if cur_mesh.is_some() {
                commands.entity(node_ent).remove::<Mesh3d>();
            }
            continue;
        };

        let Ok(registry) = registries.get(hsd_child.doc) else {
            continue;
        };

        let mesh_ent = {
            let meshes = registry.0.meshes.lock().expect("meshes lock");
            meshes
                .get(mesh_id)
                .and_then(|inner| *inner.entity.lock().expect("entity lock"))
        };
        let Some(mesh_ent) = mesh_ent else {
            continue;
        };

        let Ok(compiled_mesh) = compiled_meshes.get(mesh_ent) else {
            // Wait for mesh to compile.
            commands.entity(node_ent).insert(Uncompiled);
            continue;
        };

        let mut ent = commands.entity(node_ent);
        ent.insert(Mesh3d(compiled_mesh.0.clone()));

        // Apply compiled material or default.
        let mat_ent = mat_ref.and_then(|MaterialRef(mat_id)| {
            let mats = registry.0.materials.lock().expect("materials lock");
            mats.get(mat_id)
                .and_then(|inner| *inner.entity.lock().expect("entity lock"))
        });

        if let Some(mat_ent) = mat_ent {
            if let Ok(compiled_mat) = compiled_mats.get(mat_ent) {
                ent.insert(MeshMaterial3d(compiled_mat.0.clone()));
            } else {
                // Wait for material to compile.
                ent.insert(Uncompiled);
            }
            continue;
        }

        let mat = default_material
            .get_or_insert_with(|| asset_server.add(StandardMaterial::default()))
            .clone();
        ent.insert(MeshMaterial3d(mat));
    }
}
