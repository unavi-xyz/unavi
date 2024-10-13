use std::sync::mpsc::{Receiver, Sender, SyncSender};

use bevy::prelude::*;
use wasm_bridge::component::{Resource, ResourceTable, ResourceTableError};
use wasm_bridge_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::api::{id::UniqueId, wired::scene::nodes::base::NodeRes, ApiData};

pub type ScriptCommand = Box<dyn FnOnce(&mut World) + Send>;
pub type CommandSender = SyncSender<ScriptCommand>;

pub enum ScriptControl {
    /// Unrecoverable error.
    /// Stops script execution.
    Exit(String),
}
pub type ControlSender = Sender<ScriptControl>;

pub struct ScriptData {
    pub api: ApiData,
    pub command_recv: Receiver<ScriptCommand>,
    pub command_send: CommandSender,
    pub control_recv: Receiver<ScriptControl>,
    pub control_send: ControlSender,
    pub id: UniqueId,
    pub table: ResourceTable,
    pub wasi: WasiCtx,
    pub wasi_table: wasm_bridge_wasi::ResourceTable,
}

impl WasiView for ScriptData {
    fn table(&mut self) -> &mut wasm_bridge_wasi::ResourceTable {
        &mut self.wasi_table
    }

    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl Default for ScriptData {
    fn default() -> Self {
        // Set a hard limit on the command queue length. If this is reached,
        // script execution will crash, and the ECS may not be cleaned up, as
        // cleanup methods require sending commands through this queue.
        //
        // This should be fine, as all ECS data from a script is stored within
        // that script's scene, and can thus can still be tracked and cleaned up
        // on scene removal.
        //
        // TODO: Test that script ECS data is indeed within the script scene
        let (command_send, command_recv) = std::sync::mpsc::sync_channel(10_000);
        let (control_send, control_recv) = std::sync::mpsc::channel();

        Self {
            api: ApiData::default(),
            command_recv,
            command_send,
            control_recv,
            control_send,
            id: UniqueId::default(),
            table: ResourceTable::default(),
            wasi: WasiCtxBuilder::new().build(),
            wasi_table: wasm_bridge_wasi::ResourceTable::default(),
        }
    }
}

impl ScriptData {
    /// Pushes all recieved commands to the provided queue.
    pub fn push_commands(&self, queue: &mut Commands) {
        while let Ok(command) = self.command_recv.try_recv() {
            queue.push(command);
        }
    }

    pub fn clone_res<T: Clone + Send + 'static>(
        &mut self,
        res: &Resource<T>,
    ) -> Result<Resource<T>, ResourceTableError> {
        let data = self.table.get(res)?.clone();
        let new_res = self.table.push(data)?;
        Ok(new_res)
    }

    /// Inserts a component into the given node if the value is `Some`.
    /// If the value is `None`, removes the component from the entity.
    pub fn node_insert_option<T: Bundle>(&mut self, node: NodeRes, value: Option<T>) {
        self.command_send
            .try_send(Box::new(move |world: &mut World| {
                let entity = *node.read().entity.get().unwrap();

                let mut entity = world.entity_mut(entity);

                if let Some(value) = value {
                    entity.insert(value);
                } else {
                    entity.remove::<T>();
                }
            }))
            .unwrap();
    }

    /// Inserts a component into the given node.
    pub fn node_insert<T: Bundle>(&mut self, node: NodeRes, value: T) {
        self.node_insert_option(node, Some(value))
    }

    /// Removes a component from the given node.
    pub fn node_remove<T: Bundle>(&mut self, node: NodeRes) {
        self.node_insert_option::<T>(node, None)
    }
}
