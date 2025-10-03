use std::{alloc::Layout, ptr::NonNull};

use bevy::{
    ecs::component::{ComponentCloneBehavior, ComponentDescriptor, ComponentId, StorageType},
    prelude::*,
    ptr::OwningPtr,
};
use system::VSystemDependencies;
use tokio::sync::mpsc::error::TryRecvError;

use crate::{
    api::wired::ecs::wired::ecs::host_api::SystemOrder,
    execute::init::InitializedScript,
    load::{LoadedScript, ScriptCommands, bindings::wired::ecs::types::System as WSystem},
};

pub(crate) mod system;

#[derive(Debug)]
pub enum WasmCommand {
    RegisterComponent {
        id: u32,
        key: String,
        size: usize,
    },
    RegisterSystem {
        id: u32,
        system: WSystem,
    },
    OrderSystems {
        a: u32,
        order: SystemOrder,
        b: u32,
    },
    Spawn {
        id: u64,
    },
    Despawn {
        id: u64,
    },
    WriteComponent {
        entity_id: u64,
        component_id: u32,
        data: Vec<u8>,
        insert: bool,
    },
    RemoveComponent {
        entity_id: u64,
        component_id: u32,
    },
}

/// "Virtual" objects owned by a script.
/// These will be automatically cleaned up on script removal.
#[derive(Component, Default)]
#[relationship_target(relationship = VOwner)]
pub struct VObjects(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = VObjects)]
pub struct VOwner(Entity);

#[derive(Component)]
pub struct VEntity {
    pub wasm_id: u64,
}

#[derive(Component)]
pub struct VComponent {
    bevy_id: ComponentId,
    wasm_id: u32,
}

const BATCH_SIZE: usize = 32;

// TODO: apply commands after each script cycle (in case of multiple fixed virtual frames)
// TODO: limit script execution until full cycle completes

pub fn apply_wasm_commands(
    mut commands: Commands,
    mut script_commands: Query<(Entity, &mut ScriptCommands), With<InitializedScript>>,
    mut queue: Local<Vec<(Entity, WasmCommand)>>,
) {
    // info!("apply_wasm_commands");

    loop {
        for (ent, mut recv_commands) in script_commands.iter_mut() {
            loop {
                if queue.len() >= BATCH_SIZE {
                    break;
                }

                match recv_commands.0.try_recv() {
                    Ok(cmd) => queue.push((ent, cmd)),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        warn!("ScriptCommands disconnected");
                        break;
                    }
                }
            }
        }

        if queue.is_empty() {
            break;
        }

        for (script_ent, cmd) in queue.drain(..) {
            // info!("> {cmd:?}");

            match cmd {
                WasmCommand::RegisterComponent { id, key, size } => {
                    let layout = match Layout::array::<u8>(size) {
                        Ok(l) => l,
                        Err(e) => {
                            error!("Error registering component layout: {e:?}");
                            continue;
                        }
                    };

                    commands.queue(move |world: &mut World| {
                        // SAFETY: [u8] is Send + Sync
                        let bevy_id = world.register_component_with_descriptor(unsafe {
                            ComponentDescriptor::new_with_layout(
                                key,
                                StorageType::Table,
                                layout,
                                None,
                                true,
                                ComponentCloneBehavior::Default,
                            )
                        });

                        world.spawn((
                            VOwner(script_ent),
                            VComponent {
                                bevy_id,
                                wasm_id: id,
                            },
                        ));
                    });
                }
                WasmCommand::RegisterSystem { id, system } => {
                    commands.queue(move |world: &mut World| {
                        if let Err(e) = system::build_system(world, script_ent, id, system) {
                            error!("Error building system {id}: {e:?}");
                        };
                    });
                }
                WasmCommand::OrderSystems { a, order, b } => {
                    commands.queue(move |world: &mut World| {
                        let Some(mut vsystem_deps) =
                            world.get_mut::<VSystemDependencies>(script_ent)
                        else {
                            warn!("VSystemDependencies not found on script ent");
                            return;
                        };

                        let (up, down) = match order {
                            SystemOrder::After => (b, a),
                            SystemOrder::Before => (a, b),
                        };

                        // TODO: Only add deps to a's schedule.
                        for schedule_deps in vsystem_deps.0.values_mut() {
                            let deps = schedule_deps.dependencies.entry(down).or_default();
                            deps.push(up);
                        }
                    });
                }
                WasmCommand::Spawn { id } => {
                    commands.spawn((VOwner(script_ent), VEntity { wasm_id: id }));
                }
                WasmCommand::Despawn { id } => {
                    commands.queue(move |world: &mut World| {
                        let Some(e) = world
                            .query::<(Entity, &VOwner, &VEntity)>()
                            .iter(world)
                            .find_map(|(e, e_owner, v_ent)| {
                                if e_owner.0 == script_ent && v_ent.wasm_id == id {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                        else {
                            error!("Despawn: entity {id} not found");
                            return;
                        };

                        world.despawn(e);
                    });
                }
                WasmCommand::WriteComponent {
                    entity_id,
                    component_id,
                    mut data,
                    insert,
                } => {
                    commands.queue(move |world: &mut World| {
                        let Some(e) = world
                            .query::<(Entity, &VOwner, &VEntity)>()
                            .iter(world)
                            .find_map(|(e, e_owner, v_ent)| {
                                if e_owner.0 == script_ent && v_ent.wasm_id == entity_id {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                        else {
                            error!("InsertComponent: entity {entity_id} not found");
                            return;
                        };

                        let Some(bevy_id) = world
                            .query::<(&VOwner, &VComponent)>()
                            .iter(world)
                            .find_map(|(c_owner, v_comp)| {
                                if c_owner.0 == script_ent && v_comp.wasm_id == component_id {
                                    Some(v_comp.bevy_id)
                                } else {
                                    None
                                }
                            })
                        else {
                            error!("InsertComponent: component {component_id} not found");
                            return;
                        };

                        let Some(info) = world.components().get_info(bevy_id) else {
                            error!("InsertComponent: component info not found");
                            return;
                        };
                        let size = info.layout().size() / size_of::<u8>();
                        data.resize(size, 0);

                        let mut ent = world.entity_mut(e);
                        let has_component = ent.contains_id(bevy_id);

                        if !has_component && !insert {
                            return;
                        }

                        let ptr = data.as_mut_ptr();

                        // SAFETY:
                        // - Component ids have been taken from the same world
                        // - Each array is created to the layout specified in the world
                        unsafe {
                            let ptr = OwningPtr::new(NonNull::new_unchecked(ptr.cast()));
                            ent.insert_by_id(bevy_id, ptr);
                        }
                    });
                }
                WasmCommand::RemoveComponent {
                    entity_id,
                    component_id,
                } => {
                    commands.queue(move |world: &mut World| {
                        let Some(e) = world
                            .query::<(Entity, &VOwner, &VEntity)>()
                            .iter(world)
                            .find_map(|(e, e_owner, v_ent)| {
                                if e_owner.0 == script_ent && v_ent.wasm_id == entity_id {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                        else {
                            error!("RemoveComponent: entity {entity_id} not found");
                            return;
                        };

                        let Some(bevy_id) = world
                            .query::<(&VOwner, &VComponent)>()
                            .iter(world)
                            .find_map(|(c_owner, v_comp)| {
                                if c_owner.0 == script_ent && v_comp.wasm_id == component_id {
                                    Some(v_comp.bevy_id)
                                } else {
                                    None
                                }
                            })
                        else {
                            error!("RemoveComponent: component {component_id} not found");
                            return;
                        };

                        world.entity_mut(e).remove_by_id(bevy_id);
                    });
                }
            }
        }
    }
}

pub fn cleanup_vobjects(trigger: Trigger<OnRemove, LoadedScript>, mut commands: Commands) {
    let ent = trigger.target();
    info!("Cleaning up vobjects for {ent}");
    commands.entity(ent).despawn_related::<VObjects>();
}
