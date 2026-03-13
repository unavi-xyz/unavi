use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use loro::TreeID;
use smol_str::SmolStr;

use crate::{
    cache::{MaterialState, MeshState},
    data::{HsdMaterial, HsdMesh, HsdNodeData},
};

/// Raw entry from the Loro subscription callback (IDs only, no Loro reads).
#[derive(Debug)]
pub(crate) enum RawHsdChange {
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
pub struct DocChangeQueue(pub Arc<Mutex<Vec<DocChange>>>);

pub enum MeshData {
    Hsd(HsdMesh),
    Inline(MeshState),
}

pub enum MaterialData {
    Hsd(Box<HsdMaterial>),
    Inline(MaterialState),
}

pub enum DocChangeKind {
    NodeAdded {
        id: SmolStr,
        parent_id: Option<SmolStr>,
        data: HsdNodeData,
    },
    NodeChanged {
        id: SmolStr,
        data: HsdNodeData,
    },
    NodeParentChanged {
        id: SmolStr,
        parent_id: Option<SmolStr>,
    },
    NodeRemoved {
        id: SmolStr,
    },
    MeshAdded {
        id: SmolStr,
        data: MeshData,
    },
    MeshChanged {
        id: SmolStr,
        data: MeshData,
    },
    MeshRemoved {
        id: SmolStr,
    },
    MaterialAdded {
        id: SmolStr,
        data: MaterialData,
    },
    MaterialChanged {
        id: SmolStr,
        data: MaterialData,
    },
    MaterialRemoved {
        id: SmolStr,
    },
}

pub struct DocChange {
    pub doc: Entity,
    pub kind: DocChangeKind,
}
