use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use crossbeam::channel::Receiver;

pub mod handler;

#[derive(Component, Deref, DerefMut)]
pub struct ActionReceiver(pub Receiver<ScriptAction>);

#[derive(Debug)]
pub enum ScriptAction {
    CreateMaterial {
        id: u32,
    },
    CreateMesh {
        id: u32,
    },
    CreateNode {
        id: u32,
    },
    CreatePrimitive {
        id: u32,
        mesh: u32,
    },
    RemoveMaterial {
        id: u32,
    },
    RemoveMesh {
        id: u32,
    },
    RemoveNode {
        id: u32,
    },
    RemovePrimitive {
        id: u32,
        mesh: u32,
    },
    SetMaterialColor {
        id: u32,
        color: Color,
    },
    SetNodeCollider {
        id: u32,
        collider: Option<Collider>,
    },
    SetNodeInputHandler {
        id: u32,
        handler: Option<()>,
    },
    SetNodeMesh {
        id: u32,
        mesh: Option<u32>,
    },
    SetNodeParent {
        id: u32,
        parent: Option<u32>,
    },
    SetNodeRigidBody {
        id: u32,
        rigid_body: Option<RigidBody>,
    },
    SetNodeTransform {
        id: u32,
        transform: Transform,
    },
    SetPrimitiveIndices {
        id: u32,
        value: Vec<u32>,
    },
    SetPrimitiveMaterial {
        id: u32,
        material: Option<u32>,
    },
    SetPrimitiveNormals {
        id: u32,
        value: Vec<f32>,
    },
    SetPrimitivePositions {
        id: u32,
        value: Vec<f32>,
    },
    SetPrimitiveUvs {
        id: u32,
        value: Vec<f32>,
    },
}
