use bevy::prelude::*;
use smol_str::SmolStr;

pub mod local;
#[cfg(not(target_family = "wasm"))]
pub mod native;

/// Links a script entity back to its HSD node.
#[derive(Component)]
pub struct HsdScriptSource {
    pub node_entity: Entity,
    pub doc_entity: Entity,
    pub node_id: SmolStr,
}
