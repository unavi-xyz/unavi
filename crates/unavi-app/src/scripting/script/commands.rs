use std::alloc::Layout;

use bevy::{
    ecs::component::{ComponentDescriptor, ComponentId, StorageType},
    prelude::*,
    utils::HashMap,
};

use super::ScriptCommandReceiver;

#[derive(Debug)]
pub enum ScriptCommand {
    RegisterComponent(u32),
    RegisterQuery(RegisterQuery),
    SpawnEntity(SpawnEntity),
}

#[derive(Debug)]
pub struct ComponentInstance {
    pub component: u32,
    pub instance: u32,
}

#[derive(Debug)]
pub struct RegisterQuery {
    pub components: Vec<u32>,
    pub id: u32,
}

#[derive(Debug)]
pub struct SpawnEntity {
    pub components: Vec<ComponentInstance>,
    pub id: u32,
}

/// Maps script resources to their Bevy counterparts.
#[derive(Component, Debug, Default)]
pub struct ScriptResourceMap {
    components: HashMap<u32, ComponentId>,
    entities: HashMap<u32, Entity>,
    queries: HashMap<u32, QueryState<()>>,
}

#[derive(Debug)]
enum CommandPreprocess {
    RegisterComponent(u32),
    RegisterQuery {
        id: u32,
        components: Vec<ComponentId>,
    },
    SpawnEntity {
        id: u32,
        components: Vec<ComponentInstance>,
    },
}

enum CommandResult {
    RegisterComponent { id: u32, component: ComponentId },
    RegisterQuery { id: u32, query: Box<QueryState<()>> },
    SpawnEntity { id: u32, entity: Entity },
}

pub fn handle_script_commands(
    world: &mut World,
    mut counter: Local<usize>,
    scripts: &mut QueryState<(&ScriptCommandReceiver, &mut ScriptResourceMap)>,
) {
    // Process one command from each queue in each loop.
    loop {
        let preprocessed = preprocess(world, scripts);

        // Once all queues are empty, we're done.
        if preprocessed.iter().all(|o| o.is_none()) {
            break;
        }

        // Because we need exclusive world access to process some commands, we split
        // this up into multiple steps.
        let mut results = preprocessed
            .into_iter()
            .map(|command| {
                let command = match command {
                    Some(c) => c,
                    None => return None,
                };

                debug!("Processing script command: {:?}", command);

                match command {
                    CommandPreprocess::RegisterComponent(id) => {
                        *counter += 1;

                        // SAFETY: [u64] is Send + Sync
                        let component = world.init_component_with_descriptor(unsafe {
                            ComponentDescriptor::new_with_layout(
                                format!("_GeneratedScriptComponent{}", *counter),
                                StorageType::Table,
                                Layout::array::<u64>(1).unwrap(),
                                None,
                            )
                        });

                        Some(CommandResult::RegisterComponent { id, component })
                    }
                    CommandPreprocess::RegisterQuery { id, components } => {
                        let mut builder = QueryBuilder::<()>::new(world);

                        for id in components.into_iter() {
                            builder.ref_id(id);
                        }

                        let query = Box::new(builder.build());

                        Some(CommandResult::RegisterQuery { id, query })
                    }
                    CommandPreprocess::SpawnEntity { id, components } => {
                        let entity = world.spawn(());

                        // TODO: components

                        Some(CommandResult::SpawnEntity {
                            id,
                            entity: entity.id(),
                        })
                    }
                }
            })
            .collect::<Vec<_>>()
            .into_iter();

        for (_, mut map) in scripts.iter_mut(world) {
            let results = results.next().unwrap();

            for result in results.into_iter() {
                match result {
                    CommandResult::RegisterComponent { id, component } => {
                        map.components.insert(id, component);
                    }
                    CommandResult::RegisterQuery { id, query } => {
                        map.queries.insert(id, *query);
                    }
                    CommandResult::SpawnEntity { id, entity } => {
                        map.entities.insert(id, entity);
                    }
                }
            }
        }
    }
}

fn preprocess(
    world: &mut World,
    scripts: &mut QueryState<(&ScriptCommandReceiver, &mut ScriptResourceMap)>,
) -> Vec<Option<CommandPreprocess>> {
    scripts
        .iter_mut(world)
        .map(|(receiver, map)| {
            let receiver = match receiver.0.lock() {
                Ok(r) => r,
                Err(_) => return None,
            };

            let command = match receiver.try_recv() {
                Ok(c) => c,
                Err(_) => return None,
            };

            match command {
                ScriptCommand::RegisterComponent(id) => {
                    if map.components.contains_key(&id) {
                        warn!(
                            "Component {} already registered. Ignoring script command.",
                            id
                        );
                        return None;
                    }

                    Some(CommandPreprocess::RegisterComponent(id))
                }
                ScriptCommand::RegisterQuery(RegisterQuery { id, components }) => {
                    if map.queries.contains_key(&id) {
                        warn!("Query {} already registered. Ignoring script command.", id);
                        return None;
                    }

                    let components = components
                        .iter()
                        .map(|c| map.components.get(c).map(|c| c.to_owned()))
                        .collect::<Option<Vec<_>>>();

                    if let Some(components) = components {
                        Some(CommandPreprocess::RegisterQuery {id, components})
                    } else {
                        warn!("Unable to find component while registering query {}. Ignoring script command.", id);
                        None
                    }
                }
                ScriptCommand::SpawnEntity(SpawnEntity { id, components }) => {
                    if map.entities.contains_key(&id) {
                        warn!("Entity {} already spawned. Ignoring script command.", id);
                        return None;
                    }

                    Some(CommandPreprocess::SpawnEntity { id , components })
                }
            }
        })
        .collect()
}
