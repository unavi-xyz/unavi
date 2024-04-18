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
    // queries: HashMap<u32, Query>,
}

enum CommandResult {
    RegisterComponent { id: u32, component: ComponentId },
    RegisterQuery { id: u32 },
    SpawnEntity { id: u32, entity: Entity },
}

pub fn handle_script_commands(
    world: &mut World,
    mut counter: Local<usize>,
    scripts: &mut QueryState<(&ScriptCommandReceiver, &mut ScriptResourceMap)>,
) {
    // Because we need exclusive world access to process some commands, we split
    // this up into multiple steps.
    let mut command_results = scripts
        .iter_mut(world)
        .map(|(receiver, map)| {
            let mut commands = Vec::new();

            if let Ok(receiver) = receiver.0.lock() {
                while let Ok(command) = receiver.try_recv() {
                    // Validate command.
                    match &command {
                        ScriptCommand::RegisterComponent(id) => {
                            if map.components.contains_key(id) {
                                warn!(
                                    "Component {} already registered. Ignoring script command.",
                                    id
                                );
                                continue;
                            }
                        }
                        ScriptCommand::SpawnEntity(SpawnEntity { components, id }) => {
                            if map.entities.contains_key(id) {
                                warn!("Entity {} already spawned. Ignoring script command.", id);
                                continue;
                            }
                        }
                        _ => {}
                    }

                    commands.push(command);
                }
            };

            commands
        })
        .collect::<Vec<_>>()
        .iter()
        .map(|commands| {
            commands
                .iter()
                .map(|command| {
                    debug!("Processing script command: {:?}", command);

                    match command {
                        ScriptCommand::RegisterQuery(RegisterQuery { components, id }) => {
                            // TODO
                            CommandResult::RegisterQuery { id: *id }
                        }
                        ScriptCommand::RegisterComponent(id) => {
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

                            CommandResult::RegisterComponent { id: *id, component }
                        }
                        ScriptCommand::SpawnEntity(SpawnEntity { components, id }) => {
                            let entity = world.spawn(());
                            CommandResult::SpawnEntity {
                                id: *id,
                                entity: entity.id(),
                            }
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .into_iter();

    for (_, mut map) in scripts.iter_mut(world) {
        let results = command_results.next().unwrap();

        for result in results.into_iter() {
            match result {
                CommandResult::RegisterComponent { id, component } => {
                    map.components.insert(id, component);
                }
                CommandResult::RegisterQuery { id } => {}
                CommandResult::SpawnEntity { id, entity } => {
                    map.entities.insert(id, entity);
                }
            }
        }
    }
}
