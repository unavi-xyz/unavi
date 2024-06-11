use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

#[derive(Default)]
pub struct LocalData {
    next_id: u32,
    pub nodes: HashMap<u32, NodeData>,
    pub materials: HashMap<u32, MaterialData>,
    pub meshes: HashMap<u32, MeshData>,
}

impl LocalData {
    pub fn new_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[derive(Default)]
pub struct NodeData {
    pub children: HashSet<u32>,
    pub mesh: Option<u32>,
    pub parent: Option<u32>,
    pub resources: HashSet<u32>,
    pub transform: Transform,
}

#[derive(Default)]
pub struct MeshData {
    pub primitives: HashMap<u32, PrimitiveData>,
    pub resources: HashSet<u32>,
}

#[derive(Default)]
pub struct PrimitiveData {
    pub resources: HashSet<u32>,
}

#[derive(Default)]
pub struct MaterialData {
    pub color: Color,
    pub resources: HashSet<u32>,
}
