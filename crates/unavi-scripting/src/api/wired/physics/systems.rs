use avian3d::prelude::*;
use bevy::prelude::*;

use crate::{
    api::{wired::scene::nodes::base::NodeRef, EnvId},
    execution::ScriptTickrate,
    load::ScriptMap,
};

pub(crate) fn update_physics_transforms(
    nodes: Query<(&EnvId, &NodeRef, &Transform), With<Collider>>,
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

        let data = script.store.data();

        for (id, node, transform) in nodes.iter() {
            if id.0 != data.id {
                continue;
            }

            if let Some(node) = node.0.upgrade() {
                info!("--------- FOUND NODE -----------> {:?}", transform);
                node.write().unwrap().transform = *transform;
            }
        }
    }
}
