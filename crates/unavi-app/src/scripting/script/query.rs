use bevy::{ecs::component::ComponentId, prelude::*, tasks::futures_lite::future, utils::HashMap};
use wasm_bridge::component::Resource;

use super::{commands::ScriptResourceMap, host::wired_ecs, load::ScriptStore};

pub fn run_script_queries(
    world: &mut World,
    scripts: &mut QueryState<(&mut ScriptResourceMap, &ScriptStore)>,
) {
    let script_queries = scripts
        .iter_mut(world)
        .map(|(map, _)| {
            map.queries
                .iter()
                .map(|(id, query)| (*id, query.state.clone()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut script_results = Vec::new();

    for queries in script_queries.into_iter() {
        let mut script_queries: HashMap<u32, _> = HashMap::new();

        for (id, query) in queries.into_iter() {
            let mut query = match query.lock() {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut query_results: HashMap<Entity, _> = HashMap::new();

            for entity_ref in query.iter(world) {
                let mut queried_components: HashMap<ComponentId, u64> = HashMap::new();

                for component in entity_ref.components() {
                    let ptr = entity_ref.get_by_id(component).unwrap();

                    let info = world.components().get_info(component).unwrap();
                    let len = info.layout().size() / std::mem::size_of::<u64>();
                    let data: &mut [u64] = unsafe {
                        std::slice::from_raw_parts_mut(
                            ptr.assert_unique().as_ptr().cast::<u64>(),
                            len,
                        )
                    };

                    queried_components.insert(info.id(), *data.first().unwrap());
                }

                query_results.insert(entity_ref.id(), queried_components);
            }

            script_queries.insert(id, query_results);
        }

        script_results.push(script_queries);
    }

    for ((mut map, store), results) in scripts.iter_mut(world).zip(script_results) {
        let mut results = results
            .into_iter()
            .map(|(query_id, query_results)| {
                let query_results = query_results
                    .into_iter()
                    .map(|(entity, components)| {
                        let (entity_id, _) = map
                            .entities
                            .iter()
                            .find(|(_, ent)| **ent == entity)
                            .unwrap();

                        let components = components
                            .into_iter()
                            .map(|(_, value)| value as u32)
                            .collect::<Vec<_>>();

                        (*entity_id, components)
                    })
                    .collect::<Vec<_>>();

                (query_id, Some(query_results))
            })
            .collect::<HashMap<u32, _>>();

        for (query_id, _) in map.queries.iter_mut() {
            let query_results = results.get_mut(query_id).unwrap().take().unwrap();

            let mut store = future::block_on(async { store.0.lock().await });

            let state = store.data_mut();

            let resource = Resource::<wired_ecs::Query>::new_borrow(*query_id);
            let resource = state.table.get_mut(&resource).unwrap();

            resource.result = query_results;
        }
    }
}
