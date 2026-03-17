use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use smol_str::SmolStr;

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
        tree_id: loro::TreeID,
        parent_id: Option<loro::TreeID>,
    },
    NodeChanged {
        tree_id: loro::TreeID,
    },
    NodeRemoved {
        tree_id: loro::TreeID,
    },
}

#[derive(Component, Clone)]
pub struct RawChangeQueue(pub Arc<Mutex<Vec<RawHsdChange>>>);

#[derive(Component, Clone)]
pub struct ScriptEventQueue(pub Arc<Mutex<Vec<ScriptQueuedEvent>>>);

pub enum ScriptQueuedEvent {
    MaterialDespawned {
        id: SmolStr,
    },
    MaterialSpawned {
        id: SmolStr,
    },
    MeshDespawned {
        id: SmolStr,
    },
    MeshSpawned {
        id: SmolStr,
    },
    NodeDespawned {
        id: SmolStr,
    },
    NodeParentSet {
        id: SmolStr,
        parent: Option<SmolStr>,
    },
    NodeSpawned {
        id: SmolStr,
    },
}
