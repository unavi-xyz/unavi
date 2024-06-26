use bevy::{prelude::*, tasks::block_on};
use wasm_bridge::{component::ResourceAny, AsContextMut};

use crate::{host::wired_gltf::handler::NodeId, load::LoadedScript};

use super::load::Scripts;

#[derive(Component)]
pub struct FailedToInit;

#[derive(Component)]
pub struct ScriptResource(ResourceAny);

pub fn init_scripts(
    mut commands: Commands,
    mut to_init: Query<
        Entity,
        (
            With<LoadedScript>,
            Without<FailedToInit>,
            Without<ScriptResource>,
        ),
    >,
    scripts: NonSendMut<Scripts>,
) {
    for entity in to_init.iter_mut() {
        let res = block_on(async {
            let mut scripts = scripts.lock().await;
            let (script, store) = scripts.get_mut(&entity).unwrap();

            script
                .wired_script_types()
                .script()
                .call_constructor(store)
                .await
        });

        if let Err(e) = res {
            error!("Failed to construct script resource: {}", e);
            commands.entity(entity).insert(FailedToInit);
            continue;
        };

        commands.entity(entity).insert(ScriptResource(res.unwrap()));
    }
}

const UPDATE_HZ: f32 = 60.0;
const UPDATE_DELTA: f32 = 1.0 / UPDATE_HZ;

pub fn update_scripts(
    mut last_update: Local<f32>,
    mut to_update: Query<(Entity, &ScriptResource)>,
    scripts: NonSendMut<Scripts>,
    time: Res<Time>,
    mut nodes: Query<(&mut Transform, &NodeId)>,
) {
    let now = time.elapsed_seconds();
    let delta = now - *last_update;

    if delta < UPDATE_DELTA {
        return;
    }

    *last_update = now;

    for (entity, res) in to_update.iter_mut() {
        let res: anyhow::Result<_> = block_on(async {
            let mut scripts = scripts.lock().await;
            let (script, store) = scripts.get_mut(&entity).unwrap();

            script
                .wired_script_types()
                .script()
                .call_update(store.as_context_mut(), res.0, delta)
                .await?;

            let state = store.data();

            for res in state.nodes.iter() {
                let node = state.table.get(res)?;
                let transform = state.table.get(&node.transform)?;

                if transform.clean(&state.table)? {
                    let rep = res.rep();

                    let translation = state.table.get(&transform.translation)?;
                    let rotation = state.table.get(&transform.rotation)?;
                    let scale = state.table.get(&transform.scale)?;

                    if let Some(mut target) =
                        nodes
                            .iter_mut()
                            .find_map(|(t, id)| if id.0 == rep { Some(t) } else { None })
                    {
                        target.translation.clone_from(&translation.data);
                        target.rotation.clone_from(&rotation.data);
                        target.scale.clone_from(&scale.data);
                    } else {
                        warn!("Node {} not found", rep)
                    };
                }
            }

            Ok(())
        });

        if let Err(e) = res {
            error!("Failed to update script: {}", e);
        };
    }
}
