use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use bevy_vrm::BoneName;
use loro::{LoroDoc, TreeID};

/// Populated by `unavi-client` when the local avatar's bones are ready.
/// Used by `load_scripts` to give agent-permitted scripts access to the
/// agent HSD document and its bone-proxy nodes.
#[derive(Resource)]
pub struct LocalAgentHsdDoc {
    pub doc: Arc<LoroDoc>,
    /// Maps each VRM bone to its proxy root node in `doc`.
    pub bone_nodes: Arc<HashMap<BoneName, TreeID>>,
}
