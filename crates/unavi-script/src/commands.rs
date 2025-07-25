use std::alloc::Layout;

use bevy::{
    ecs::component::{ComponentCloneBehavior, ComponentDescriptor, ComponentId, StorageType},
    prelude::*,
};
use tokio::sync::mpsc::error::TryRecvError;

use crate::load::ScriptCommands;

pub enum WasmCommand {
    RegisterComponent { id: u64, key: String, size: usize },
    RegisterSystem { id: u64 },
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

const MAX_THROUGHPUT: usize = 400;

pub fn process_commands(world: &mut World, mut commands: Local<Vec<(Entity, WasmCommand)>>) {
    let mut scripts = world.query::<(Entity, &mut ScriptCommands)>();

    for (ent, mut recv) in scripts.iter_mut(world) {
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

    for (ent, cmd) in commands.drain(..) {
        match cmd {
            WasmCommand::RegisterComponent { id, key, size } => {
                debug!("Registering component {id} ({key}):\n - size={size}");

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
                    VOwner(ent),
                    VComponent {
                        bevy_id,
                        wasm_id: id,
                    },
                ));
            }
            WasmCommand::RegisterSystem { id } => {}
        }
    }
}
