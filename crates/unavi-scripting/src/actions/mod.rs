use bevy::prelude::*;
use crossbeam::channel::Receiver;

pub mod handler;

#[derive(Component, Deref, DerefMut)]
pub struct ActionReceiver(pub Receiver<ScriptAction>);

#[derive(Debug)]
pub enum ScriptAction {
    CreateMaterial { id: u32 },
    CreateMesh { id: u32 },
    CreateNode { id: u32 },
    CreatePrimitive { id: u32, mesh: u32 },
    RemoveMaterial { id: u32 },
    RemoveMesh { id: u32 },
    RemoveNode { id: u32 },
    RemovePrimitive { id: u32, mesh: u32 },
    SetMaterialColor { id: u32, color: Color },
    SetNodeMesh { id: u32, mesh: Option<u32> },
    SetNodeParent { id: u32, parent: Option<u32> },
    SetNodeTransform { id: u32, transform: Transform },
    SetPrimitiveIndices { id: u32, value: Vec<u32> },
    SetPrimitiveMaterial { id: u32, material: Option<u32> },
    SetPrimitiveNormals { id: u32, value: Vec<f32> },
    SetPrimitivePositions { id: u32, value: Vec<f32> },
    SetPrimitiveUvs { id: u32, value: Vec<f32> },
}
