use bevy::prelude::*;

use crate::cache::SceneRegistry;

pub fn sync_ecs_to_cache(registries: Query<&SceneRegistry>, transforms: Query<&GlobalTransform>) {
    for registry in &registries {
        let nodes = registry.0.nodes.lock().expect("nodes lock");
        for node_inner in nodes.iter() {
            let ent = *node_inner.entity.lock().expect("entity lock");
            let Some(ent) = ent else { continue };
            let Ok(gt) = transforms.get(ent) else {
                continue;
            };
            let mut state = node_inner.state.lock().expect("node state lock");
            state.global_transform = *gt;
        }
    }
}
