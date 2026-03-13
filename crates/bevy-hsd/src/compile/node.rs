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
            // Mesh referenced but not yet registered — wait.
            commands.entity(node_ent).insert(Uncompiled);
            continue;
        };

        let Ok(compiled_mesh) = compiled_meshes.get(mesh_ent) else {
            // Wait for mesh to compile.
            commands.entity(node_ent).insert(Uncompiled);
            continue;
        };

        // Resolve material before touching Mesh3d so both are inserted together.
        let mat = if let Some(MaterialRef(mat_id)) = mat_ref {
            let mat_ent = {
                let mats = registry.0.materials.lock().expect("materials lock");
                mats.get(mat_id)
                    .and_then(|inner| *inner.entity.lock().expect("entity lock"))
            };
            if let Some(cm) = mat_ent.and_then(|e| compiled_mats.get(e).ok()) {
                MeshMaterial3d(cm.0.clone())
            } else {
                // Material referenced but not yet registered or compiled — wait.
                commands.entity(node_ent).insert(Uncompiled);
                continue;
            }
        } else {
            // No material assigned — use default.
            MeshMaterial3d(
                default_material
                    .get_or_insert_with(|| asset_server.add(StandardMaterial::default()))
                    .clone(),
            )
        };

        commands
            .entity(node_ent)
            .insert((Mesh3d(compiled_mesh.0.clone()), mat));
    }
}
