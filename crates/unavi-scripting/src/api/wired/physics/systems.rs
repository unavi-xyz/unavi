use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    api::{
        utils::RefResource,
        wired::scene::nodes::base::{NodeId, NodeRes},
    },
    execution::ScriptTickrate,
    load::ScriptMap,
};

pub(crate) fn update_physics_transforms(
    nodes: Query<(&NodeId, Entity, &Transform), With<Collider>>,
    script_map: NonSendMut<ScriptMap>,
    scripts: Query<(Entity, &ScriptTickrate)>,
) {
    for (entity, tickrate) in scripts.iter() {
        if !tickrate.ready_for_update {
            continue;
        }

        let mut scripts = script_map.lock().unwrap();

        let Some(script) = scripts.get_mut(&entity) else {
            continue;
        };

        let data = script.store.data_mut();
        let script_nodes = data
            .api
            .wired_scene
            .as_ref()
            .unwrap()
            .entities
            .nodes
            .read()
            .unwrap();

        for (id, ent, transform) in nodes.iter() {
            // Check if phys node is from this script.
            if Some(ent) != script_nodes.get(&id.0).copied() {
                continue;
            }

            let node = NodeRes::from_rep(id.0, &data.table).unwrap();
            let node = data.table.get_mut(&node).unwrap();

            node.transform = *transform;
        }
    }
}
