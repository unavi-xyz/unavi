use std::sync::{Arc, Mutex};

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use loro::TreeID;
use smol_str::SmolStr;

#[derive(Clone, Debug)]
pub enum NodeRef {
    Entity(Entity),
    Id(SmolStr),
}

#[derive(Debug)]
pub enum RawHsdChange {
    MaterialAdded {
        id: SmolStr,
    },
    MaterialChanged {
        id: SmolStr,
    },
    MaterialRemoved {
        id: SmolStr,
    },
    MeshAdded {
        id: SmolStr,
    },
    MeshChanged {
        id: SmolStr,
    },
    MeshRemoved {
        id: SmolStr,
    },
    NodeAdded {
        tree_id: TreeID,
        parent_id: Option<TreeID>,
    },
    NodeChanged {
        tree_id: TreeID,
    },
    NodeRemoved {
        tree_id: TreeID,
    },
}

#[derive(Component, Clone)]
pub struct RawChangeQueue(pub Arc<Mutex<Vec<RawHsdChange>>>);

pub const SCRIPT_COMMAND_LIMIT: usize = 1 << 16;

/// Per-script command queue bridging WASM setters (async) to Bevy ECS (main thread).
///
/// Setters push `FnOnce(&mut World)` closures here. The queue is drained via
/// `commands.append` after each WASM call in the polling systems.
#[derive(Default)]
pub struct ScriptCommandQueue {
    pub inner: CommandQueue,
    pub len: usize,
}

impl ScriptCommandQueue {
    pub fn push<C: bevy::prelude::Command>(&mut self, cmd: C) {
        if self.len < SCRIPT_COMMAND_LIMIT {
            self.inner.push(cmd);
            self.len += 1;
        } else {
            warn_once!(
                "script command queue limit ({SCRIPT_COMMAND_LIMIT}) reached, dropping commands"
            );
        }
    }
}

#[derive(Component, Clone)]
pub struct ScriptCommandQueueComp(pub Arc<Mutex<ScriptCommandQueue>>);
