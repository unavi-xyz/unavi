use bevy::prelude::*;

use crate::{
    HsdChild, HsdScripts, MaterialRef, MeshRef, NodeId,
    data::{HsdCollider, HsdNode, HsdNodeData, HsdRigidBody},
};

pub(super) fn spawn_node_entity(
    doc_ent: Entity,
    node: &HsdNode,
    commands: &mut Commands,
) -> Entity {
    let transform = node_transform(&node.data);

    let mut ent = commands.spawn((
        HsdChild { doc: doc_ent },
        NodeId(node.id.clone()),
        transform,
    ));

    if let Some(id) = node.data.mesh.clone() {
        ent.insert(MeshRef(id));
    }
    if let Some(id) = node.data.material.clone() {
        ent.insert(MaterialRef(id));
    }
    if let Some(collider) = &node.data.collider {
        ent.insert(HsdCollider::clone(collider));
    }
    if let Some(rb) = &node.data.rigid_body {
        ent.insert(HsdRigidBody::clone(rb));
    }
    if let Some(scripts) = &node.data.scripts
        && !scripts.is_empty()
    {
        let hashes: Vec<blake3::Hash> = scripts.iter().map(|h| h.0).collect();
        ent.insert(HsdScripts(hashes));
    }

    ent.id()
}

pub(super) fn update_node_components(
    ent: Entity,
    id: &str,
    data: &HsdNodeData,
    commands: &mut Commands,
) {
    let transform = node_transform(data);

    let mut ecmd = commands.entity(ent);
    ecmd.insert((NodeId(id.into()), transform));
    ecmd.remove::<(MeshRef, MaterialRef)>();

    if let Some(id) = data.mesh.clone() {
        ecmd.insert(MeshRef(id));
    }
    if let Some(id) = data.material.clone() {
        ecmd.insert(MaterialRef(id));
    }

    let hashes: Vec<blake3::Hash> = data
        .scripts
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|h| h.0)
        .collect();
    if hashes.is_empty() {
        ecmd.remove::<HsdScripts>();
    } else {
        ecmd.insert(HsdScripts(hashes));
    }
}

pub(super) fn node_transform(data: &HsdNodeData) -> Transform {
    let mut t = Transform::default();
    if let Some(tr) = &data.translation
        && tr.len() >= 3
    {
        #[expect(clippy::cast_possible_truncation)]
        {
            t.translation = Vec3::new(tr[0] as f32, tr[1] as f32, tr[2] as f32);
        }
    }
    if let Some(r) = &data.rotation
        && r.len() >= 4
    {
        #[expect(clippy::cast_possible_truncation)]
        {
            t.rotation = Quat::from_xyzw(r[0] as f32, r[1] as f32, r[2] as f32, r[3] as f32);
        }
    }
    if let Some(s) = &data.scale
        && s.len() >= 3
    {
        #[expect(clippy::cast_possible_truncation)]
        {
            t.scale = Vec3::new(s[0] as f32, s[1] as f32, s[2] as f32);
        }
    }
    t
}
