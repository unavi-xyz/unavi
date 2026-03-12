use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use loro::TreeID;
use smol_str::SmolStr;

use crate::data::{HsdCollider, HsdRigidBody};

pub struct NodeState {
    pub name: Option<String>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub mesh: Option<SmolStr>,
    pub material: Option<SmolStr>,
    pub collider: Option<HsdCollider>,
    pub rigid_body: Option<HsdRigidBody>,
    pub scripts: Vec<blake3::Hash>,
    /// Weak back-reference to avoid reference cycles.
    pub parent: Option<Weak<NodeInner>>,
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
    pub state: Mutex<NodeState>,
    pub entity: Mutex<Option<Entity>>,
    pub tree_id: TreeID,
}

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
    pub state: Mutex<MeshState>,
    pub entity: Mutex<Option<Entity>>,
    pub id: SmolStr,
}

pub struct MaterialState {
    pub name: Option<String>,
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub double_sided: bool,
}

impl Default for MaterialState {
    fn default() -> Self {
        Self {
            name: None,
            base_color: [1.0, 1.0, 1.0, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            double_sided: false,
        }
    }
}

pub struct MaterialInner {
    pub state: Mutex<MaterialState>,
    pub entity: Mutex<Option<Entity>>,
    pub id: SmolStr,
}

pub struct SceneRegistryInner {
    pub nodes: Mutex<Vec<Arc<NodeInner>>>,
    pub node_map: Mutex<HashMap<TreeID, Arc<NodeInner>>>,
    pub meshes: Mutex<HashMap<SmolStr, Arc<MeshInner>>>,
    pub materials: Mutex<HashMap<SmolStr, Arc<MaterialInner>>>,
}

impl SceneRegistryInner {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            nodes: Mutex::new(Vec::new()),
            node_map: Mutex::new(HashMap::new()),
            meshes: Mutex::new(HashMap::new()),
            materials: Mutex::new(HashMap::new()),
        })
    }
}

impl Default for SceneRegistryInner {
    fn default() -> Self {
        Self {
            nodes: Mutex::new(Vec::new()),
            node_map: Mutex::new(HashMap::new()),
            meshes: Mutex::new(HashMap::new()),
            materials: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Component, Clone)]
pub struct SceneRegistry(pub Arc<SceneRegistryInner>);

pub enum SceneEvent {
    NodeCreated(Arc<NodeInner>),
    NodeDirty(Arc<NodeInner>),
    NodeParentChanged {
        node: Arc<NodeInner>,
        parent: Option<Arc<NodeInner>>,
    },
    NodeDestroyed(Arc<NodeInner>),
    MeshCreated(Arc<MeshInner>),
    MeshDirty(Arc<MeshInner>),
    MaterialCreated(Arc<MaterialInner>),
    MaterialDirty(Arc<MaterialInner>),
}

#[derive(Component, Clone)]
pub struct SceneEventQueue(pub Arc<Mutex<Vec<SceneEvent>>>);
