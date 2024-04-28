use std::{alloc::Layout, ptr::NonNull};

use bevy::{
    ecs::{
        component::{ComponentDescriptor, ComponentInfo, StorageType},
        world::FilteredEntityRef,
    },
    prelude::*,
    ptr::OwningPtr,
    utils::{HashMap, HashSet},
};
use crossbeam::channel::{Receiver, Sender};

use super::{QueriedEntity, WiredEcsCommand, WiredEcsReceiver};

#[derive(Component)]
pub struct WiredEcsMap {
    pub components: HashMap<u32, ComponentInfo>,
    pub entities: HashMap<u32, Entity>,
    pub queries: HashSet<u32>,
    pub query_receiver: Receiver<WiredQuery>,
    pub query_receiver_results: Receiver<WiredQuery>,
    pub query_sender: Sender<WiredQuery>,
    pub query_sender_results: Sender<WiredQuery>,
}

impl Default for WiredEcsMap {
    fn default() -> Self {
        let (query_sender, query_receiver) = crossbeam::channel::unbounded();
        let (query_sender_results, query_receiver_results) = crossbeam::channel::unbounded();

        Self {
            components: Default::default(),
            entities: Default::default(),
            queries: Default::default(),
            query_receiver,
            query_receiver_results,
            query_sender,
            query_sender_results,
        }
    }
}

pub struct WiredQuery {
    pub id: u32,
    pub result: Vec<QueriedEntity>,
    pub state: QueryState<FilteredEntityRef<'static>>,
}

pub fn add_wired_ecs_map(
    mut commands: Commands,
    scripts: Query<Entity, (With<WiredEcsReceiver>, Without<WiredEcsMap>)>,
) {
    for entity in scripts.iter() {
        commands.entity(entity).insert(WiredEcsMap::default());
    }
}

pub fn handle_wired_ecs_command(
    world: &mut World,
    mut counter: Local<usize>,
    scripts: &mut QueryState<(&WiredEcsReceiver, &mut WiredEcsMap)>,
) {
    let receivers = scripts
        .iter(world)
        .map(|(r, _)| r.0.clone())
        .collect::<Vec<_>>();

    for (i, receiver) in receivers.iter().enumerate() {
        while let Ok(command) = receiver.try_recv() {
            match command {
                WiredEcsCommand::Insert {
                    entity: _,
                    instance: _,
                } => {}
                WiredEcsCommand::RegisterComponent { id } => {
                    let (_, map) = scripts.iter(world).nth(i).unwrap();

                    if map.components.contains_key(&id) {
                        warn!("Component {} already registered. Ignoring command.", id);
                        return;
                    }

                    *counter += 1;

                    // SAFETY: Copied from Bevy idk 🔥🔥🔥
                    let component_id = world.init_component_with_descriptor(unsafe {
                        ComponentDescriptor::new_with_layout(
                            format!("_GeneratedScriptComponent{}", *counter),
                            StorageType::Table,
                            Layout::array::<u64>(1).unwrap(),
                            None,
                        )
                    });

                    let info = world.components().get_info(component_id).unwrap().clone();

                    let (_, mut map) = scripts.iter_mut(world).nth(i).unwrap();
                    map.components.insert(id, info);
                }
                WiredEcsCommand::RegisterQuery { id, components } => {
                    let (_, map) = scripts.iter(world).nth(i).unwrap();

                    if map.queries.contains(&id) {
                        warn!("Query {} already registered. Ignoring command.", id);
                        return;
                    }

                    let mut info_ids = Vec::new();

                    for component in components.into_iter() {
                        match map.components.get(&component) {
                            Some(info) => info_ids.push(info.id()),
                            None => {
                                error!("Component {} not found.", component);
                                continue;
                            }
                        };
                    }

                    let mut builder = QueryBuilder::new(world);

                    for id in info_ids {
                        builder.ref_id(id);
                    }

                    let state = builder.build();
                    let query = WiredQuery {
                        id,
                        result: Default::default(),
                        state,
                    };

                    let (_, mut map) = scripts.iter_mut(world).nth(i).unwrap();

                    if let Err(e) = map.query_sender.send(query) {
                        error!("Failed to send query: {}", e);
                        continue;
                    }

                    map.queries.insert(id);
                }
                WiredEcsCommand::Spawn { id, components } => {
                    let (_, map) = scripts.iter(world).nth(i).unwrap();

                    if map.entities.contains_key(&id) {
                        warn!("Entity {} already spawned. Ignoring command.", id);
                        return;
                    }

                    let mut infos = Vec::new();

                    for component in components.into_iter() {
                        match map.components.get(&component.component) {
                            Some(info) => infos.push((info.clone(), component)),
                            None => {
                                error!("Component {} not found.", component.component);
                                continue;
                            }
                        }
                    }

                    let mut entity = world.spawn(());

                    for (info, instance) in infos.into_iter() {
                        let len = info.layout().size() / std::mem::size_of::<u64>();
                        let mut data = vec![instance.id as u64; len];
                        let ptr = data.as_mut_ptr();

                        // SAFETY: `data` needs to match the component layout.
                        // We get `len` from `info` so this is always the case.
                        unsafe {
                            let non_null = NonNull::new_unchecked(ptr.cast());
                            let owning_ptr = OwningPtr::new(non_null);
                            entity.insert_by_id(info.id(), owning_ptr);
                        };
                    }

                    let entity = entity.id();

                    let (_, mut map) = scripts.iter_mut(world).nth(i).unwrap();
                    map.entities.insert(id, entity);
                }
            }
        }
    }
}