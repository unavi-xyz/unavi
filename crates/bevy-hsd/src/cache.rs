use std::{
    collections::HashMap,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use loro::TreeID;
use smol_str::SmolStr;

use crate::data::{HsdCollider, HsdRigidBody};

#[expect(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct NodeDirty {
    pub collider: bool,
    pub material: bool,
    pub mesh: bool,
    pub name: bool,
    pub rigid_body: bool,
    pub transform: bool,
}

impl NodeDirty {
    #[must_use]
    pub const fn any(&self) -> bool {
        self.collider
            || self.material
            || self.mesh
            || self.name
            || self.rigid_body
            || self.transform
    }
}

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

#[derive(Default)]
pub struct MeshDirty {
    pub geometry: bool,
}

impl MeshDirty {
    #[must_use]
    pub const fn any(&self) -> bool {
        self.geometry
    }
}

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

#[expect(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct MaterialDirty {
    pub alpha_cutoff: bool,
    pub alpha_mode: bool,
    pub base_color: bool,
    pub double_sided: bool,
    pub metallic: bool,
    pub name: bool,
    pub roughness: bool,
}

impl MaterialDirty {
    #[must_use]
    pub const fn any(&self) -> bool {
        self.alpha_cutoff
            || self.alpha_mode
            || self.base_color
            || self.double_sided
            || self.metallic
            || self.name
            || self.roughness
    }
}

#[derive(Default)]
pub struct MaterialHsdChanges {
    pub alpha_cutoff: Option<f64>,
    pub alpha_mode: Option<Option<String>>,
    pub base_color: Option<[f64; 4]>,
    pub double_sided: Option<bool>,
    pub metallic: Option<f64>,
    pub name: Option<Option<String>>,
    pub roughness: Option<f64>,
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
    }
}

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

pub struct NodeInner {
    pub dirty: Mutex<NodeDirty>,
    pub entity: Mutex<Option<Entity>>,
    pub hsd_changes: Mutex<NodeHsdChanges>,
    pub id: SmolStr,
    pub is_virtual: bool,
    pub state: Mutex<NodeState>,
    pub sync: AtomicBool,
    pub tree_id: Mutex<Option<TreeID>>,
}

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

pub struct MeshInner {
    pub dirty: Mutex<MeshDirty>,
    pub entity: Mutex<Option<Entity>>,
    pub hsd_changes: Mutex<MeshHsdChanges>,
    pub id: SmolStr,
    pub state: Mutex<MeshState>,
    pub sync: AtomicBool,
}

#[derive(Clone)]
pub struct MaterialState {
    pub alpha_cutoff: Option<f32>,
    pub alpha_mode: Option<String>,
    pub base_color: [f32; 4],
    pub double_sided: bool,
    pub metallic: f32,
    pub name: Option<String>,
    pub roughness: f32,
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
        }
    }
}

pub struct MaterialInner {
    pub dirty: Mutex<MaterialDirty>,
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

pub struct SceneRegistryInner {
    pub doc_sync: AtomicBool,
    pub materials: Mutex<HashMap<SmolStr, Arc<MaterialInner>>>,
    pub meshes: Mutex<HashMap<SmolStr, Arc<MeshInner>>>,
    pub node_map: Mutex<HashMap<SmolStr, Arc<NodeInner>>>,
    pub nodes: Mutex<Vec<Arc<NodeInner>>>,
    pub pending_doc_ops: Mutex<Vec<SyncOp>>,
}

impl SceneRegistryInner {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            doc_sync: false.into(),
            materials: Mutex::new(HashMap::new()),
            meshes: Mutex::new(HashMap::new()),
            node_map: Mutex::new(HashMap::new()),
            nodes: Mutex::new(Vec::new()),
            pending_doc_ops: Mutex::new(Vec::new()),
        })
    }
}

impl Default for SceneRegistryInner {
    fn default() -> Self {
        Self {
            doc_sync: false.into(),
            materials: Mutex::new(HashMap::new()),
            meshes: Mutex::new(HashMap::new()),
            node_map: Mutex::new(HashMap::new()),
            nodes: Mutex::new(Vec::new()),
            pending_doc_ops: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Component, Clone)]
pub struct SceneRegistry(pub Arc<SceneRegistryInner>);
