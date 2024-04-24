use std::{alloc::Layout, ptr::NonNull, sync::Arc};

use bevy::{
    ecs::{
        component::{ComponentDescriptor, ComponentInfo, StorageType},
        world::FilteredEntityRef,
    },
    prelude::*,
    ptr::OwningPtr,
    utils::HashMap,
};
use tokio::sync::Mutex;

use crate::scripting::util::blocking_lock;

use super::{WiredEcsCommand, WiredEcsReceiver};

/// Maps wired-ecs resources to their Bevy counterparts.
#[derive(Component, Debug, Default)]
pub struct WiredEcsMap {
    pub components: HashMap<u32, ComponentInfo>,
    pub entities: HashMap<u32, Entity>,
    pub queries: HashMap<u32, Arc<Mutex<QueryState<FilteredEntityRef<'static>>>>>,
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
        let receiver = blocking_lock(receiver);

        while let Ok(command) = receiver.try_recv() {
            match command {
                WiredEcsCommand::Insert { entity, instance } => {}
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

                    if map.queries.contains_key(&id) {
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

                    let query = builder.build();

                    let (_, mut map) = scripts.iter_mut(world).nth(i).unwrap();
                    map.queries.insert(id, Arc::new(Mutex::new(query)));
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
                        // We get `len` from `info` so this should be fine.
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
