//! Shared event and queue types for the hydration pipeline.

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

/// Coarse change from a Loro diff. Field-level events are emitted later in
/// `process_hsd_queue` after re-hydrating the object.
#[derive(Debug)]
pub enum RawHsdChange {
    ImageAdded {
        id: SmolStr,
    },
    ImageChanged {
        id: SmolStr,
    },
    ImageRemoved {
        id: SmolStr,
    },
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

/// Per-doc queue written by the Loro subscription thread, drained each FixedUpdate.
#[derive(Component, Clone)]
pub struct RawChangeQueue(pub Arc<Mutex<Vec<RawHsdChange>>>);

pub const SCRIPT_COMMAND_LIMIT: usize = 1 << 16;

/// Per-script command queue bridging WASM setters (async) to Bevy ECS (main thread).
///
/// Setters push `FnOnce(&mut World)` closures here. The queue is drained via
/// `commands.append` after each WASM call in the polling systems.
/// One-way bridge from WASM script callbacks (async) into Bevy Commands (main
/// thread). Capped at `SCRIPT_COMMAND_LIMIT` to prevent runaway scripts.
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
