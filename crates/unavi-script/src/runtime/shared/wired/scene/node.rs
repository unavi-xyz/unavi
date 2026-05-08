use std::sync::Arc;

use bevy::math::{Quat, Vec3};
use blake3::Hash;
use hsd::{HsdCollider, HsdNode, HsdRigidBody};
use loro::{LoroDoc, LoroMap, LoroTree, TreeID, TreeParentId};
use loro_surgeon::{Hydrate, Reconcile};
use smol_str::SmolStr;

use crate::{
    firewall::Channel,
    runtime::shared::{
        Api,
        registry::{
            firewall::validate_firewall,
            transform::{AbsoluteNodeId, NODE_TRANSFORM_REGISTRY},
        },
        wired::scene::{
            material::MaterialRes,
            mesh::MeshRes,
            util::{bytes_to_f32s, bytes_to_u32s, f32s_to_bytes, u32s_to_bytes},
        },
    },
};

#[derive(Clone)]
pub struct NodeRes {
    pub doc: Arc<LoroDoc>,
    pub doc_id: Hash,
    pub id: TreeID,
}

pub struct NodeTransform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for NodeTransform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        }
    }
}

fn get_node(api: &Api, rep: u32) -> anyhow::Result<NodeRes> {
    api.wired_scene
        .try_lock()?
        .nodes
        .get(rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid node rep: {rep}"))
}

fn node_tree(doc: &LoroDoc) -> anyhow::Result<LoroTree> {
    doc.get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .map_err(Into::into)
}

fn node_meta(tree: &LoroTree, id: TreeID) -> anyhow::Result<LoroMap> {
    tree.get_meta(id).map_err(Into::into)
}

fn hydrate_node(meta: &LoroMap) -> HsdNode {
    HsdNode::hydrate(&meta.get_deep_value()).unwrap_or_default()
}

pub fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .nodes
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid node"))
}

pub fn on_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.nodes.remove(rep);
    Ok(())
}

pub fn id(api: &Api, rep: u32) -> anyhow::Result<String> {
    let node = get_node(api, rep)?;
    Ok(node.id.to_string())
}

pub fn name(api: &Api, rep: u32) -> anyhow::Result<Option<String>> {
    let node = get_node(api, rep)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    Ok(hydrate_node(&meta).name.map(|s| s.to_string()))
}

pub fn set_name(api: &Api, rep: u32, value: Option<String>) -> anyhow::Result<()> {
    let node = get_node(api, rep)?;
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.name = value.map(SmolStr::from);
    data.reconcile(&meta)?;
    Ok(())
}

pub fn translation(api: &Api, rep: u32) -> anyhow::Result<[f32; 3]> {
    let node = get_node(api, rep)?;
    let tr = NODE_TRANSFORM_REGISTRY
        .get_sync(&AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        })
        .map(|v| v.get().local)
        .unwrap_or_default();
    Ok(tr.translation.to_array())
}

pub fn set_translation(api: &Api, rep: u32, value: [f32; 3]) -> anyhow::Result<()> {
    let node = get_node(api, rep)?;
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;

    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.translation = Some(value.iter().map(|&v| f64::from(v)).collect());
    data.reconcile(&meta)?;

    NODE_TRANSFORM_REGISTRY.update_sync(
        &AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        },
        |_, v| {
            v.local.translation = Vec3::from_array(value);
        },
    );

    Ok(())
}

pub fn rotation(api: &Api, rep: u32) -> anyhow::Result<[f32; 4]> {
    let node = get_node(api, rep)?;
    let ro = NODE_TRANSFORM_REGISTRY
        .get_sync(&AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        })
        .map(|v| v.get().local)
        .unwrap_or_default();
    let q = ro.rotation;
    Ok([q.x, q.y, q.z, q.w])
}

pub fn set_rotation(api: &Api, rep: u32, value: [f32; 4]) -> anyhow::Result<()> {
    let node = get_node(api, rep)?;
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;

    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.rotation = Some(value.iter().map(|&v| f64::from(v)).collect());
    data.reconcile(&meta)?;

    NODE_TRANSFORM_REGISTRY.update_sync(
        &AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        },
        |_, v| {
            v.local.rotation = Quat::from_array(value);
        },
    );

    Ok(())
}

pub fn scale(api: &Api, rep: u32) -> anyhow::Result<[f32; 3]> {
    let node = get_node(api, rep)?;
    let sc = NODE_TRANSFORM_REGISTRY
        .get_sync(&AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        })
        .map(|v| v.get().local)
        .unwrap_or_default();
    Ok(sc.scale.to_array())
}

pub fn set_scale(api: &Api, rep: u32, value: [f32; 3]) -> anyhow::Result<()> {
    let node = get_node(api, rep)?;
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;

    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.scale = Some(value.iter().map(|&v| f64::from(v)).collect());
    data.reconcile(&meta)?;

    NODE_TRANSFORM_REGISTRY.update_sync(
        &AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        },
        |_, v| {
            v.local.scale = Vec3::from_array(value);
        },
    );

    Ok(())
}

pub fn transform(api: &Api, rep: u32) -> anyhow::Result<NodeTransform> {
    let node = get_node(api, rep)?;
    let local = NODE_TRANSFORM_REGISTRY
        .get_sync(&AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        })
        .map(|v| v.get().local)
        .unwrap_or_default();
    let q = local.rotation;
    Ok(NodeTransform {
        translation: local.translation.to_array(),
        rotation: [q.x, q.y, q.z, q.w],
        scale: local.scale.to_array(),
    })
}

pub fn set_transform(api: &Api, rep: u32, value: NodeTransform) -> anyhow::Result<()> {
    let node = get_node(api, rep)?;
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;

    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.translation = Some(value.translation.iter().map(|&v| f64::from(v)).collect());
    data.rotation = Some(value.rotation.iter().map(|&v| f64::from(v)).collect());
    data.scale = Some(value.scale.iter().map(|&v| f64::from(v)).collect());
    data.reconcile(&meta)?;

    NODE_TRANSFORM_REGISTRY.update_sync(
        &AbsoluteNodeId {
            doc: node.doc_id,
            node: node.id,
        },
        |_, v| {
            v.local.translation = Vec3::from_array(value.translation);
            v.local.rotation = Quat::from_array(value.rotation);
            v.local.scale = Vec3::from_array(value.scale);
        },
    );

    Ok(())
}

pub fn global_transform(api: &Api, rep: u32) -> anyhow::Result<NodeTransform> {
    let node = get_node(api, rep)?;
    let key = AbsoluteNodeId {
        doc: node.doc_id,
        node: node.id,
    };
    let snapshot = NODE_TRANSFORM_REGISTRY
        .read_sync(&key, |_, v| v.clone())
        .unwrap_or_default();
    let (sc, ro, tr) = snapshot.global.to_scale_rotation_translation();
    Ok(NodeTransform {
        translation: [tr.x, tr.y, tr.z],
        rotation: [ro.x, ro.y, ro.z, ro.w],
        scale: [sc.x, sc.y, sc.z],
    })
}

pub fn parent(api: &Api, rep: u32) -> anyhow::Result<Option<u32>> {
    let node = get_node(api, rep)?;
    let tree = node_tree(&node.doc)?;
    let Some(TreeParentId::Node(parent_id)) = tree.parent(node.id) else {
        return Ok(None);
    };
    let mut scene = api.wired_scene.try_lock()?;
    Ok(Some(scene.nodes.insert(NodeRes {
        doc: node.doc,
        doc_id: node.doc_id,
        id: parent_id,
    })))
}

pub fn children(api: &Api, rep: u32) -> anyhow::Result<Vec<u32>> {
    let node = get_node(api, rep)?;
    let tree = node_tree(&node.doc)?;
    let child_ids = tree.children(node.id).unwrap_or_default();
    let mut scene = api.wired_scene.try_lock()?;
    Ok(child_ids
        .into_iter()
        .map(|id| {
            scene.nodes.insert(NodeRes {
                doc: Arc::clone(&node.doc),
                doc_id: node.doc_id,
                id,
            })
        })
        .collect())
}

pub fn add_child(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<()> {
    let scene = api.wired_scene.try_lock()?;
    let parent = scene
        .nodes
        .get(self_rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid parent rep: {self_rep}"))?;
    let child = scene
        .nodes
        .get(child_rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid child rep: {child_rep}"))?;
    drop(scene);

    validate_firewall(&api.doc_id, &parent.doc_id, Channel::SceneWrite)?;
    anyhow::ensure!(
        Arc::ptr_eq(&parent.doc, &child.doc),
        "nodes must belong to the same document"
    );
    let tree = node_tree(&parent.doc)?;
    tree.mov_to(child.id, TreeParentId::Node(parent.id), usize::MAX)?;
    Ok(())
}

pub fn remove_child(api: &Api, self_rep: u32, child_rep: u32) -> anyhow::Result<()> {
    let scene = api.wired_scene.try_lock()?;
    let parent = scene
        .nodes
        .get(self_rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid parent rep: {self_rep}"))?;
    let child = scene
        .nodes
        .get(child_rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid child rep: {child_rep}"))?;
    drop(scene);

    validate_firewall(&api.doc_id, &parent.doc_id, Channel::SceneWrite)?;
    let tree = node_tree(&child.doc)?;
    tree.mov_to(child.id, TreeParentId::Root, usize::MAX)?;
    Ok(())
}

pub fn mesh(api: &Api, rep: u32) -> anyhow::Result<Option<u32>> {
    let node = get_node(api, rep)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let Some(mesh_id) = hydrate_node(&meta).mesh else {
        return Ok(None);
    };
    let mut scene = api.wired_scene.try_lock()?;
    if let Some(key) = scene
        .meshes
        .items
        .iter()
        .find(|(_, m)| m.doc_id == node.doc_id && m.id == mesh_id)
        .map(|(k, _)| *k)
    {
        return Ok(scene.meshes.insert_clone(key));
    }
    Ok(Some(scene.meshes.insert(MeshRes {
        doc: Arc::clone(&node.doc),
        doc_id: node.doc_id,
        id: mesh_id,
    })))
}

pub fn set_mesh(api: &Api, node_rep: u32, mesh_rep: Option<u32>) -> anyhow::Result<()> {
    let scene = api.wired_scene.try_lock()?;
    let mesh_id = if let Some(mrep) = mesh_rep {
        let m = scene
            .meshes
            .get(mrep)
            .ok_or_else(|| anyhow::anyhow!("invalid mesh rep: {mrep}"))?;
        Some(m.id.clone())
    } else {
        None
    };
    let node = scene
        .nodes
        .get(node_rep)
        .ok_or_else(|| anyhow::anyhow!("invalid node rep: {node_rep}"))?;

    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    drop(scene);
    let mut data = hydrate_node(&meta);
    data.mesh = mesh_id;
    data.reconcile(&meta)?;
    Ok(())
}

pub fn material(api: &Api, rep: u32) -> anyhow::Result<Option<u32>> {
    let node = get_node(api, rep)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let Some(mat_id) = hydrate_node(&meta).material else {
        return Ok(None);
    };
    let mut scene = api.wired_scene.try_lock()?;
    if let Some(key) = scene
        .materials
        .items
        .iter()
        .find(|(_, m)| m.doc_id == node.doc_id && m.id == mat_id)
        .map(|(k, _)| *k)
    {
        return Ok(scene.materials.insert_clone(key));
    }
    Ok(Some(scene.materials.insert(MaterialRes {
        doc: Arc::clone(&node.doc),
        doc_id: node.doc_id,
        id: mat_id,
    })))
}

pub enum NodeCollider {
    Capsule {
        height: f32,
        radius: f32,
    },
    ConvexHull(Vec<f32>),
    Cuboid([f32; 3]),
    Cylinder {
        height: f32,
        radius: f32,
    },
    Sphere(f32),
    Trimesh {
        indices: Vec<u32>,
        vertices: Vec<f32>,
    },
}

pub enum NodeRigidBody {
    Dynamic,
    Fixed,
    Kinematic,
}

pub async fn collider(api: &Api, rep: u32) -> anyhow::Result<Option<NodeCollider>> {
    let node = get_node(api, rep)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let Some(c) = hydrate_node(&meta).collider else {
        return Ok(None);
    };
    Ok(match c {
        HsdCollider::Capsule { height, radius } => Some(NodeCollider::Capsule {
            height: height as f32,
            radius: radius as f32,
        }),
        HsdCollider::ConvexHull(hash) => {
            let bytes = super::fetch_blob(hash.into()).await?;
            Some(NodeCollider::ConvexHull(bytes_to_f32s(&bytes)))
        }
        HsdCollider::Cuboid { x, y, z } => {
            Some(NodeCollider::Cuboid([x as f32, y as f32, z as f32]))
        }
        HsdCollider::Cylinder { height, radius } => Some(NodeCollider::Cylinder {
            height: height as f32,
            radius: radius as f32,
        }),
        HsdCollider::Sphere(r) => Some(NodeCollider::Sphere(r as f32)),
        HsdCollider::Trimesh { indices, vertices } => {
            let idx_bytes = super::fetch_blob(indices.into()).await?;
            let vert_bytes = super::fetch_blob(vertices.into()).await?;
            Some(NodeCollider::Trimesh {
                indices: bytes_to_u32s(&idx_bytes),
                vertices: bytes_to_f32s(&vert_bytes),
            })
        }
    })
}

pub async fn set_collider(api: &Api, rep: u32, value: Option<NodeCollider>) -> anyhow::Result<()> {
    let node = get_node(api, rep)?;
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.collider = match value {
        None => None,
        Some(NodeCollider::Capsule { height, radius }) => Some(HsdCollider::Capsule {
            height: f64::from(height),
            radius: f64::from(radius),
        }),
        Some(NodeCollider::ConvexHull(points)) => {
            let hash = super::upload_blob(f32s_to_bytes(&points)).await?;
            Some(HsdCollider::ConvexHull(hash.into()))
        }
        Some(NodeCollider::Cuboid([x, y, z])) => Some(HsdCollider::Cuboid {
            x: f64::from(x),
            y: f64::from(y),
            z: f64::from(z),
        }),
        Some(NodeCollider::Cylinder { height, radius }) => Some(HsdCollider::Cylinder {
            height: f64::from(height),
            radius: f64::from(radius),
        }),
        Some(NodeCollider::Sphere(r)) => Some(HsdCollider::Sphere(f64::from(r))),
        Some(NodeCollider::Trimesh { indices, vertices }) => {
            let idx_hash = super::upload_blob(u32s_to_bytes(&indices)).await?;
            let vert_hash = super::upload_blob(f32s_to_bytes(&vertices)).await?;
            Some(HsdCollider::Trimesh {
                indices: idx_hash.into(),
                vertices: vert_hash.into(),
            })
        }
    };
    data.reconcile(&meta)?;
    Ok(())
}

pub fn rigid_body(api: &Api, rep: u32) -> anyhow::Result<Option<NodeRigidBody>> {
    let node = get_node(api, rep)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let Some(rb) = hydrate_node(&meta).rigid_body else {
        return Ok(None);
    };
    Ok(match rb.kind.as_str() {
        "dynamic" => Some(NodeRigidBody::Dynamic),
        "fixed" => Some(NodeRigidBody::Fixed),
        "kinematic" => Some(NodeRigidBody::Kinematic),
        _ => None,
    })
}

pub fn set_rigid_body(api: &Api, rep: u32, value: Option<NodeRigidBody>) -> anyhow::Result<()> {
    let node = get_node(api, rep)?;
    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.rigid_body = value.map(|rb| HsdRigidBody {
        kind: SmolStr::from(match rb {
            NodeRigidBody::Dynamic => "dynamic",
            NodeRigidBody::Fixed => "fixed",
            NodeRigidBody::Kinematic => "kinematic",
        }),
        ..Default::default()
    });
    data.reconcile(&meta)?;
    Ok(())
}

pub fn set_material(api: &Api, node_rep: u32, mat_rep: Option<u32>) -> anyhow::Result<()> {
    let scene = api.wired_scene.try_lock()?;
    let node = scene
        .nodes
        .get(node_rep)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("invalid node rep: {node_rep}"))?;
    let mat_id = if let Some(mrep) = mat_rep {
        let m = scene
            .materials
            .get(mrep)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("invalid material rep: {mrep}"))?;
        Some(m.id)
    } else {
        None
    };
    drop(scene);

    validate_firewall(&api.doc_id, &node.doc_id, Channel::SceneWrite)?;
    let tree = node_tree(&node.doc)?;
    let meta = node_meta(&tree, node.id)?;
    let mut data = hydrate_node(&meta);
    data.material = mat_id;
    data.reconcile(&meta)?;
    Ok(())
}
