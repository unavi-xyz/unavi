use anyhow::Result;
use bevy::prelude::*;
use crossbeam::channel::Receiver;
use wasm_component_layer::{Linker, Store};

use crate::{load::EngineBackend, StoreData};

mod mesh;
mod node;
pub mod system;

#[derive(Component, Deref, DerefMut)]
pub struct WiredGltfReceiver(pub Receiver<WiredGltfAction>);

pub enum WiredGltfAction {
    CreateNode { id: u32 },
    RemoveNode { id: u32 },
}

pub fn add_to_host(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<WiredGltfReceiver> {
    let (send, recv) = crossbeam::channel::bounded::<WiredGltfAction>(100);

    mesh::add_to_host(store, linker, send.clone())?;
    node::add_to_host(store, linker, send)?;

    Ok(WiredGltfReceiver(recv))
}
