use std::{
    alloc::Layout,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use bevy::{
    ecs::{
        component::{ComponentDescriptor, ComponentInfo, StorageType},
        world::FilteredEntityRef,
    },
    prelude::*,
    ptr::OwningPtr,
    tasks::futures_lite::future,
    utils::HashMap,
};

use super::load::ScriptCommandReceiver;

#[derive(Debug)]
pub enum ScriptCommand {
    RegisterComponent(u32),
    RegisterQuery(RegisterQuery),
    SpawnEntity(SpawnEntity),
}

#[derive(Debug)]
pub struct ComponentInstance {
    pub component: u32,
    pub instance: u64,
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
    pub components: HashMap<u32, ComponentInfo>,
    pub entities: HashMap<u32, Entity>,
    pub queries: HashMap<u32, ScriptQuery>,
}

#[derive(Debug)]
pub struct ScriptQuery {
    pub state: Arc<Mutex<QueryState<FilteredEntityRef<'static>>>>,
}

#[derive(Debug)]
enum CommandPreprocess {
    RegisterComponent(u32),
    RegisterQuery {
        id: u32,
        components: Vec<ComponentInfo>,
    },
    SpawnEntity {
        id: u32,
        components: Vec<(u64, ComponentInfo)>,
    },
}

enum CommandResult {
    RegisterComponent {
        id: u32,
        component: ComponentInfo,
    },
    RegisterQuery {
        id: u32,
        query: Box<QueryState<FilteredEntityRef<'static>>>,
    },
    SpawnEntity {
        id: u32,
        entity: Entity,
    },
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

                        let component_id = world.init_component_with_descriptor(unsafe {
                            ComponentDescriptor::new_with_layout(
                                format!("_GeneratedScriptComponent{}", *counter),
                                StorageType::Table,
                                Layout::array::<u64>(1).unwrap(),
                                None,
                            )
                        });

                        let info = world.components().get_info(component_id).unwrap();

                        Some(CommandResult::RegisterComponent {
                            id,
                            component: info.clone(),
                        })
                    }
                    CommandPreprocess::RegisterQuery { id, components } => {
                        let mut builder = QueryBuilder::new(world);

                        for info in components.into_iter() {
                            builder.ref_id(info.id());
                        }

                        let query = Box::new(builder.build());

                        Some(CommandResult::RegisterQuery { id, query })
                    }
                    CommandPreprocess::SpawnEntity { id, components } => {
                        let mut entity = world.spawn(());

                        for (instance, info) in components {
                            let len = info.layout().size() / std::mem::size_of::<u64>();
                            let mut data = vec![instance; len];
                            let ptr = data.as_mut_ptr();

                            unsafe {
                                let non_null = NonNull::new_unchecked(ptr.cast());
                                let owning_ptr = OwningPtr::new(non_null);
                                entity.insert_by_id(info.id(), owning_ptr);
                            };
                        }

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
                        map.queries.insert(
                            id,
                            ScriptQuery {
                                state: Arc::new(Mutex::new(*query)),
                            },
                        );
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
            let receiver = future::block_on(async {
                receiver.0.lock().await
            });


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

                    let components = components.iter().map(|item| {
                        map.components.get(&item.component).cloned().map(|info| (item.instance, info))
                    }).collect::<Option<Vec<_>>>();

                    if let Some(components) = components {
                        Some(CommandPreprocess::SpawnEntity { id , components })
                    }  else {
                        warn!("Unable to find component for spawning entity {}. Ignoring script command.", id );
                        None
                    }
                }
            }
        })
        .collect()
}
