use bevy::prelude::*;

use crate::Owner;

use super::{handler::NodeId, WiredGltfData};

pub fn query_node_data(
    nodes: Query<(&Owner, &NodeId, &Transform)>,
    scripts: Query<(Entity, &WiredGltfData)>,
) {
    for (entity, data) in scripts.iter() {
        let mut data = match data.write() {
            Ok(d) => d,
            Err(e) => {
                error!("Failed to lock data: {}", e);
                continue;
            }
        };

        for (owner, id, transform) in nodes.iter() {
            if owner.0 != entity {
                continue;
            }

            if let Some(found) = data.nodes.get_mut(&id.0) {
                found.clone_from(transform);
            } else {
                data.nodes.insert(id.0, *transform);
            }
        }
    }
}
