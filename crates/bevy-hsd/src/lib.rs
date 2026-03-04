use bevy::prelude::*;
use bevy::transform::TransformSystems;
use loro::LoroDoc;
use std::sync::Arc;

pub mod cache;
mod compile;
pub mod data;
pub mod hydrate;

pub use cache::{
    MaterialInner, MaterialState, MeshInner, MeshState, NodeInner, NodeState, SceneEvent,
    SceneEventQueue, SceneRegistry, SceneRegistryInner,
};

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
        tree_id: smol_str::SmolStr,
        parent_id: Option<smol_str::SmolStr>,
    },
    NodeMetaChanged {
        tree_id: smol_str::SmolStr,
    },
    NodeRemoved {
        tree_id: smol_str::SmolStr,
    },
}

pub struct HsdPlugin;

impl Plugin for HsdPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                hydrate::systems::init_hsd_doc,
                hydrate::systems::apply_scene_events,
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
        )
        .add_systems(
            PostUpdate,
            hydrate::systems::sync_ecs_to_cache.after(TransformSystems::Propagate),
        );
    }
}

#[derive(Component)]
#[require(HsdChildren)]
pub struct HsdDoc(pub Arc<LoroDoc>);

/// Relationship for all HSD-spawned entities.
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
pub struct NodeId(pub smol_str::SmolStr);

/// Stable Loro tree ID on node entities. Format: "counter@peer".
#[derive(Component, Clone, Debug)]
pub struct HsdNodeTreeId(pub smol_str::SmolStr);

/// Script blob hashes on node entities.
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

/// Keeps the doc subscription alive.
#[derive(Component)]
pub struct HsdSubscription(pub loro::Subscription);
