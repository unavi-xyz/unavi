use std::alloc::Layout;

use bevy::{
    ecs::{
        component::{ComponentCloneBehavior, ComponentDescriptor, ComponentId, StorageType},
        system::SystemId,
    },
    prelude::*,
};
use tokio::sync::mpsc::error::TryRecvError;

use crate::{
    execute::init::InitializedScript,
    load::{LoadedScript, ScriptCommands, bindings::wired::ecs::types::Schedule as BSchedule},
};

pub(crate) mod system;

pub enum WasmCommand {
    RegisterComponent { id: u64, key: String, size: usize },
    RegisterSystem { id: u64, schedule: BSchedule },
}

/// "Virtual" objects owned by a script.
/// These will be automatically cleaned up on script removal.
#[derive(Component, Default)]
#[relationship_target(relationship = VOwner)]
struct VObjects(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = VObjects)]
struct VOwner(Entity);

#[derive(Component)]
struct VComponent {
    bevy_id: ComponentId,
    wasm_id: u64,
}

#[derive(Component)]
struct VSystem {
    bevy_id: SystemId,
    wasm_id: u64,
}

const MAX_THROUGHPUT: usize = 400;

// TODO: process commands immuediately after system execution
// use deferred bevy commands?

pub fn process_commands(
    world: &mut World,
    mut script_commands: Local<QueryState<(Entity, &mut ScriptCommands), With<InitializedScript>>>,
    mut commands: Local<Vec<(Entity, WasmCommand)>>,
) {
    for (ent, mut recv) in script_commands.iter_mut(world) {
        loop {
            if commands.len() >= MAX_THROUGHPUT {
                break;
            }

            match recv.0.try_recv() {
                Ok(cmd) => commands.push((ent, cmd)),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    warn!("ScriptCommands disconnected");
                    break;
                }
            }
        }
    }

    for (entity, cmd) in commands.drain(..) {
        match cmd {
            WasmCommand::RegisterComponent { id, key, size } => {
                info!("Registering component {id} ({key}): size={size}");

                let layout = match Layout::array::<u8>(size) {
                    Ok(l) => l,
                    Err(e) => {
                        error!("Error registering component layout: {e:?}");
                        continue;
                    }
                };

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
                    VOwner(entity),
                    VComponent {
                        bevy_id,
                        wasm_id: id,
                    },
                ));
            }
            WasmCommand::RegisterSystem { id, schedule } => {
                info!("Registering system {id} with {schedule:?}");

                system::build_system(world, entity, id, schedule);

                // world.spawn((
                //     VOwner(ent),
                //     VSystem {
                //         bevy_id,
                //         wasm_id: id,
                //     },
                //     SystemExecution::default(),
                // ));
            }
        }
    }
}

pub fn cleanup_vobjects(trigger: Trigger<OnRemove, LoadedScript>, mut commands: Commands) {
    let ent = trigger.target();
    info!("Cleaning up vobjects for {ent}");
    commands.entity(ent).despawn_related::<VObjects>();
}
