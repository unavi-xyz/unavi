use std::sync::{Arc, RwLock};

use anyhow::Result;
use bevy::{prelude::*, utils::HashMap};
use crossbeam::channel::Receiver;
use wasm_bridge::component::Linker;

use crate::State;

pub mod bindgen;
mod node;

#[derive(Component, Deref, DerefMut)]
pub struct WiredGltfReceiver(pub Receiver<WiredGltfAction>);

#[derive(Debug)]
pub enum WiredGltfAction {
    CreateMaterial { id: u32 },
    CreateMesh { id: u32 },
    CreateNode { id: u32 },
    CreatePrimitive { id: u32, mesh: u32 },
    RemoveMaterial { id: u32 },
    RemoveMesh { id: u32 },
    RemoveNode { id: u32 },
    RemovePrimitive { id: u32 },
    SetMaterialColor { id: u32, color: Color },
    SetNodeMesh { id: u32, mesh: Option<u32> },
    SetNodeParent { id: u32, parent: Option<u32> },
    SetNodeTransform { id: u32, transform: Transform },
    SetPrimitiveIndices { id: u32, value: Vec<u32> },
    SetPrimitiveMaterial { id: u32, material: u32 },
    SetPrimitiveNormals { id: u32, value: Vec<f32> },
    SetPrimitivePositions { id: u32, value: Vec<f32> },
    SetPrimitiveUvs { id: u32, value: Vec<f32> },
}

#[derive(Component, Deref, DerefMut)]
pub struct WiredGltfData(pub Arc<RwLock<EcsData>>);

#[derive(Default)]
pub struct EcsData {
    nodes: HashMap<u32, Transform>,
}

pub fn add_to_host(linker: &mut Linker<State>) -> Result<()> {
    bindgen::wired::gltf::node::add_to_linker(linker, |s| s)?;
    Ok(())
}
