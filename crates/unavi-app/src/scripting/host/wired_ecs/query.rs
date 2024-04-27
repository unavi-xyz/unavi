use bevy::prelude::*;

use crate::scripting::load::WasmStore;

use super::{QueriedEntity, WiredEcsMap};

#[derive(Component)]
pub struct QueryFuture;

pub fn run_script_queries(
    world: &mut World,
    scripts: &mut QueryState<(&mut WiredEcsMap, &WasmStore), Without<QueryFuture>>,
) {
    let mut receivers = Vec::new();
    let mut senders = Vec::new();

    for (map, _) in scripts.iter_mut(world) {
        receivers.push(map.query_receiver.clone());
        senders.push(map.query_sender_results.clone());
    }

    for (i, (receiver, sender)) in receivers.into_iter().zip(senders).enumerate() {
        while let Ok(mut query) = receiver.try_recv() {
            query.result = Vec::new();

            for entity in query.state.iter(world) {
                let mut instances = Vec::new();

                for component in entity.components() {
                    let ptr = entity.get_by_id(component).unwrap();

                    let info = world.components().get_info(component).unwrap();
                    let len = info.layout().size() / std::mem::size_of::<u64>();
                    let data: &mut [u64] = unsafe {
                        std::slice::from_raw_parts_mut(
                            ptr.assert_unique().as_ptr().cast::<u64>(),
                            len,
                        )
                    };

                    instances.push(data[0] as u32);
                }

                let (map, _) = scripts.iter(world).nth(i).unwrap();

                if let Some((id, _)) = map.entities.iter().find(|(_, ent)| **ent == entity.id()) {
                    query.result.push(QueriedEntity {
                        entity: *id,
                        instances,
                    });
                } else {
                    warn!("Queried entity not found: {}", entity.id().index());
                }
            }

            // Send query to out channel.
            if let Err(e) = sender.send(query) {
                error!("Failed to send query: {}", e);
                continue;
            }
        }
    }
}
