use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    HsdChild, HsdChildren, HsdDoc, HsdMaterialEntities, HsdMeshEntities, HsdVersion, MaterialRef,
    MeshRef, NodeId,
    data::{HsdCollider, HsdRigidBody, hydrate_hsd},
};

pub fn hydrate_hsd_docs(mut commands: Commands, docs: Query<(Entity, &HsdDoc, &mut HsdVersion)>) {
    for (doc_ent, doc, mut version) in docs {
        let frontiers = doc.0.oplog_frontiers();

        if let Some(prev) = &version.0
            && *prev == frontiers
        {
            continue;
        }

        // Despawn all existing children (clean slate).
        commands.entity(doc_ent).despawn_related::<HsdChildren>();

        let hsd_map = doc.0.get_map("hsd");

        let hsd_data = match hydrate_hsd(&hsd_map) {
            Ok(d) => d,
            Err(err) => {
                warn!(?err, "failed to hydrate hsd doc");
                continue;
            }
        };

        // Spawn material entities.
        let mut mat_entities = Vec::with_capacity(hsd_data.materials.len());
        for mat in &hsd_data.materials {
            let ent = commands
                .spawn((
                    HsdChild { doc: doc_ent },
                    crate::data::HsdMaterial::clone(mat),
                ))
                .id();
            mat_entities.push(ent);
        }

        // Spawn mesh entities.
        let mut mesh_entities = Vec::with_capacity(hsd_data.meshes.len());
        for mesh in &hsd_data.meshes {
            let ent = commands
                .spawn((HsdChild { doc: doc_ent }, crate::data::HsdMesh::clone(mesh)))
                .id();
            mesh_entities.push(ent);
        }

        // Spawn node entities, tracking tree_id -> entity.
        let mut node_map = HashMap::new();

        for node in &hsd_data.nodes {
            let mut transform = Transform::default();

            if let Some(t) = &node.data.translation
                && t.len() >= 3
            {
                #[expect(clippy::cast_possible_truncation)]
                let v = Vec3::new(t[0] as f32, t[1] as f32, t[2] as f32);
                transform.translation = v;
            }
            if let Some(r) = &node.data.rotation
                && r.len() >= 4
            {
                #[expect(clippy::cast_possible_truncation)]
                let q = Quat::from_xyzw(r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32);
                transform.rotation = q;
            }
            if let Some(s) = &node.data.scale
                && s.len() >= 3
            {
                #[expect(clippy::cast_possible_truncation)]
                let v = Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32);
                transform.scale = v;
            }

            let name = node
                .data
                .name
                .clone()
                .unwrap_or_else(|| node.tree_id.as_str().into());

            let mut ent = commands.spawn((HsdChild { doc: doc_ent }, NodeId(name), transform));

            if let Some(idx) = node.data.mesh
                && let Ok(idx) = usize::try_from(idx)
            {
                ent.insert(MeshRef(idx));
            }
            if let Some(idx) = node.data.material
                && let Ok(idx) = usize::try_from(idx)
            {
                ent.insert(MaterialRef(idx));
            }
            if let Some(collider) = &node.data.collider {
                ent.insert(HsdCollider::clone(collider));
            }
            if let Some(rb) = &node.data.rigid_body {
                ent.insert(HsdRigidBody::clone(rb));
            }

            node_map.insert(node.tree_id.clone(), ent.id());
        }

        // Set parent/child relationships.
        for node in &hsd_data.nodes {
            if let Some(parent_id) = &node.parent_tree_id
                && let Some(&parent_ent) = node_map.get(parent_id)
                && let Some(&child_ent) = node_map.get(&node.tree_id)
            {
                commands.entity(child_ent).insert(ChildOf(parent_ent));
            }
        }

        // Store resource entity lists on doc entity.
        commands.entity(doc_ent).insert((
            HsdMeshEntities(mesh_entities),
            HsdMaterialEntities(mat_entities),
        ));

        version.0 = Some(frontiers);
    }
}
