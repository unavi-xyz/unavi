use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use loro::LoroDoc;
use smol_str::SmolStr;

mod compile;
pub mod data;
mod hydrate;

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                hydrate::init_script_overlay,
                hydrate::init_hsd_doc,
                hydrate::apply_hsd_events,
                (
                    compile::material::parse_material_data,
                    compile::mesh::parse_mesh_data,
                    compile::collider::parse_collider_data,
                    compile::rigid_body::parse_rigid_body_data,
                ),
                (
                    compile::material::compile_materials,
                    compile::mesh::compile_meshes,
                ),
                compile::compile_nodes,
            )
                .chain(),
        );
    }
}

/// Source-of-truth HSD document. Insert on an entity to create an HSD
/// scene. All child entities are cleaned up when removed.
#[derive(Component)]
#[require(HsdChildren)]
pub struct HsdDoc(pub Arc<LoroDoc>);

/// Relationship target for all HSD-spawned entities.
#[derive(Component, Default)]
#[relationship_target(relationship = HsdChild, linked_spawn)]
pub struct HsdChildren(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = HsdChildren)]
pub struct HsdChild {
    #[relationship]
    pub doc: Entity,
}

#[derive(Component, Clone, Debug)]
pub struct NodeId(pub SmolStr);

/// Stable Loro tree ID on node entities. Format: "counter@peer".
#[derive(Component, Clone, Debug)]
pub struct HsdNodeTreeId(pub SmolStr);

/// Script blob hashes on node entities (Phase 3).
#[derive(Component, Clone, Debug)]
pub struct HsdScripts(pub Vec<blake3::Hash>);

#[derive(Component)]
pub struct MeshRef(pub usize);

#[derive(Component)]
pub struct MaterialRef(pub usize);

#[derive(Component)]
pub struct CompiledMesh(pub Handle<Mesh>);

#[derive(Component)]
pub struct CompiledMaterial(pub Handle<StandardMaterial>);

/// Stable mapping from Loro container IDs to Bevy entities.
#[derive(Component, Default)]
pub struct HsdEntityMap {
    pub meshes: Vec<Option<Entity>>,
    pub materials: Vec<Option<Entity>>,
    pub nodes: HashMap<SmolStr, Entity>,
}

#[derive(Component, Clone)]
pub struct HsdEventQueue(pub Arc<Mutex<Vec<HsdChange>>>);

/// Keeps the base-to-overlay subscription alive.
#[derive(Component)]
pub struct HsdBaseSubscription(pub loro::Subscription);

/// Local-only render doc: copy of `HsdDoc` + future script writes.
/// Never exported for sync.
#[derive(Component)]
pub struct HsdScriptOverlay(pub Arc<LoroDoc>);

/// Keeps the overlay subscription alive.
#[derive(Component)]
pub struct HsdSubscription(pub loro::Subscription);

pub use hydrate::{apply_hsd_events, init_hsd_doc, init_script_overlay};

pub enum HsdChange {
    MaterialAdded,
    MaterialChanged {
        index: usize,
    },
    MaterialRemoved {
        index: usize,
    },
    MeshAdded,
    MeshChanged {
        index: usize,
    },
    MeshRemoved {
        index: usize,
    },
    NodeAdded {
        tree_id: SmolStr,
        parent_id: Option<SmolStr>,
    },
    NodeMetaChanged {
        tree_id: SmolStr,
    },
    NodeRemoved {
        tree_id: SmolStr,
    },
}
