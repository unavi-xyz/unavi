//! Per-document state that bridges the CRDT world and the ECS world.
//!
//! Each scene object (node, mesh, material, image) has an `*Inner` entry
//! mapping its HSD string ID to a Bevy entity, plus cached state used for
//! write-back from ECS back to the CRDT.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use loro::TreeID;
use smol_str::SmolStr;

use crate::data::{HsdCollider, HsdRigidBody};

/// Pending write-back fields for a node; non-empty means ECS has mutations
/// not yet committed to the CRDT.
#[derive(Default)]
pub struct NodeHsdChanges {
    pub material: Option<Option<SmolStr>>,
    pub mesh: Option<Option<SmolStr>>,
    pub name: Option<Option<String>>,
    pub rotation: Option<[f64; 4]>,
    pub scale: Option<[f64; 3]>,
    pub translation: Option<[f64; 3]>,
}

impl NodeHsdChanges {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.material.is_none()
            && self.mesh.is_none()
            && self.name.is_none()
            && self.rotation.is_none()
            && self.scale.is_none()
            && self.translation.is_none()
    }
}

/// Pending write-back fields for a mesh.
#[derive(Default)]
pub struct MeshHsdChanges {
    pub name: Option<Option<String>>,
    pub topology: Option<i64>,
}

impl MeshHsdChanges {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.name.is_none() && self.topology.is_none()
    }
}

/// Pending write-back fields for a material.
#[derive(Default)]
pub struct MaterialHsdChanges {
    pub alpha_cutoff: Option<f64>,
    pub alpha_mode: Option<Option<String>>,
    pub base_color: Option<[f64; 4]>,
    pub double_sided: Option<bool>,
    pub metallic: Option<f64>,
    pub name: Option<Option<String>>,
    pub roughness: Option<f64>,
    pub unlit: Option<bool>,
}

impl MaterialHsdChanges {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.alpha_cutoff.is_none()
            && self.alpha_mode.is_none()
            && self.base_color.is_none()
            && self.double_sided.is_none()
            && self.metallic.is_none()
            && self.name.is_none()
            && self.roughness.is_none()
            && self.unlit.is_none()
    }
}

/// Snapshot of a node's ECS state; diffed on each sync tick to detect changes.
#[derive(Clone)]
pub struct NodeState {
    pub name: Option<String>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub mesh: Option<SmolStr>,
    pub material: Option<SmolStr>,
    pub collider: Option<HsdCollider>,
    pub rigid_body: Option<HsdRigidBody>,
    pub scripts: Vec<blake3::Hash>,
    pub parent: Option<std::sync::Weak<NodeInner>>,
    pub children: Vec<Arc<NodeInner>>,
}

impl Default for NodeState {
    fn default() -> Self {
        Self {
            name: None,
            transform: Transform::IDENTITY,
            global_transform: GlobalTransform::IDENTITY,
            mesh: None,
            material: None,
            collider: None,
            rigid_body: None,
            scripts: Vec::new(),
            parent: None,
            children: Vec::new(),
        }
    }
}

/// Per-node registry entry shared between observers and sync systems.
pub struct NodeInner {
    pub entity: Mutex<Option<Entity>>,
    pub hsd_changes: Mutex<NodeHsdChanges>,
    pub id: SmolStr,
    pub is_virtual: bool,
    pub state: Mutex<NodeState>,
    pub sync: AtomicBool,
    pub tree_id: Mutex<Option<TreeID>>,
}

/// Snapshot of a mesh's decoded attribute data; used for write-back.
#[derive(Clone)]
pub struct MeshState {
    pub name: Option<String>,
    pub topology: PrimitiveTopology,
    pub indices: Option<Vec<u32>>,
    pub positions: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,
    pub tangents: Option<Vec<f32>>,
    pub colors: Option<Vec<f32>>,
    pub uv0: Option<Vec<f32>>,
    pub uv1: Option<Vec<f32>>,
}

impl Default for MeshState {
    fn default() -> Self {
        Self {
            name: None,
            topology: PrimitiveTopology::TriangleList,
            indices: None,
            positions: None,
            normals: None,
            tangents: None,
            colors: None,
            uv0: None,
            uv1: None,
        }
    }
}

/// Per-mesh registry entry.
pub struct MeshInner {
    pub entity: Mutex<Option<Entity>>,
    pub hsd_changes: Mutex<MeshHsdChanges>,
    pub id: SmolStr,
    pub state: Mutex<MeshState>,
    pub sync: AtomicBool,
}

/// Per-image registry entry.
pub struct ImageInner {
    pub entity: Mutex<Option<Entity>>,
    pub id: SmolStr,
}

/// Snapshot of a material's ECS state; used for write-back.
#[derive(Clone)]
pub struct MaterialState {
    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: Option<String>,
    pub base_color: [f32; 4],
    pub double_sided: bool,
    pub metallic: f32,
    pub name: Option<String>,
    pub roughness: f32,
    pub unlit: bool,
}

impl Default for MaterialState {
    fn default() -> Self {
        Self {
            alpha_cutoff: None,
            alpha_mode: None,
            base_color: [1.0, 1.0, 1.0, 1.0],
            double_sided: false,
            metallic: 0.0,
            name: None,
            roughness: 0.5,
            unlit: false,
        }
    }
}

/// Per-material registry entry.
pub struct MaterialInner {
    pub entity: Mutex<Option<Entity>>,
    pub hsd_changes: Mutex<MaterialHsdChanges>,
    pub id: SmolStr,
    pub state: Mutex<MaterialState>,
    pub sync: AtomicBool,
}

pub enum SyncOp {
    MaterialCreated(SmolStr),
    MaterialRemoved(SmolStr),
    MeshCreated(SmolStr),
    MeshRemoved(SmolStr),
    NodeCreated(SmolStr),
    NodeRemoved(SmolStr),
}

/// All per-doc scene state; stored on the doc entity and accessed by observers.
pub struct SceneRegistryInner {
    pub doc_sync: AtomicBool,
    pub doc_transform: Mutex<Transform>,
    pub images: Mutex<HashMap<SmolStr, Arc<ImageInner>>>,
    pub materials: Mutex<HashMap<SmolStr, Arc<MaterialInner>>>,
    pub meshes: Mutex<HashMap<SmolStr, Arc<MeshInner>>>,
    pub node_map: Mutex<HashMap<SmolStr, Arc<NodeInner>>>,
    pub nodes: Mutex<Vec<Arc<NodeInner>>>,
    pub pending_doc_ops: Mutex<Vec<SyncOp>>,
}

impl SceneRegistryInner {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }
}

impl Default for SceneRegistryInner {
    fn default() -> Self {
        Self {
            doc_sync: false.into(),
            doc_transform: Mutex::new(Transform::IDENTITY),
            images: Mutex::new(HashMap::new()),
            materials: Mutex::new(HashMap::new()),
            meshes: Mutex::new(HashMap::new()),
            node_map: Mutex::new(HashMap::new()),
            nodes: Mutex::new(Vec::new()),
            pending_doc_ops: Mutex::new(Vec::new()),
        }
    }
}

/// Cheap-to-clone handle to the doc's `SceneRegistryInner`; stored as a component.
#[derive(Component, Clone)]
pub struct SceneRegistry(pub Arc<SceneRegistryInner>);
