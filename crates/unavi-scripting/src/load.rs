use std::sync::{Arc, Mutex};

use bevy::{prelude::*, utils::HashMap};
use bevy_async_task::{AsyncTaskPool, AsyncTaskStatus};

use crate::{
    api::wired::dwn::WiredDwn,
    env::{ScriptEnv, ScriptEnvBuilder},
};

use super::asset::Wasm;

pub fn load_scripts(
    wasm_assets: Res<Assets<Wasm>>,
    default_material: Res<DefaultMaterial>,
    mut commands: Commands,
    mut pool: AsyncTaskPool<anyhow::Result<Entity>>,
    scripts: NonSendMut<ScriptMap>,
    to_load: Query<(Entity, &Name, &Handle<Wasm>, &ScriptExecutionLevel), Without<ProcessedScript>>,
) {
    for (entity, name, handle, level) in to_load.iter() {
        let wasm = match wasm_assets.get(handle) {
            Some(a) => a,
            None => continue,
        };

        info!("Loading script: {}", name);
        commands.entity(entity).insert(ProcessedScript);

        let mut builder = ScriptEnvBuilder::default();

        builder.enable_wired_input();
        builder.enable_wired_log(name.to_string());
        builder.enable_wired_physics();
        builder.enable_wired_player();
        builder.enable_wired_scene(entity, default_material.0.clone());

        if *level == ScriptExecutionLevel::System {
            // TODO
            // builder.enable_wired_dwn(WiredDwn::new(actor.0.dwn.clone()));
        }

        let bytes = wasm.0.clone();
        let scripts = scripts.clone();

        builder.data.push_commands(&mut commands);

        pool.spawn(async move {
            let script = builder.instantiate_script(&bytes).await?;

            let mut scripts = scripts.lock().unwrap();
            scripts.insert(entity, script);

            Ok(entity)
        });
    }

    for task in pool.iter_poll() {
        if let AsyncTaskStatus::Finished(res) = task {
            let entity = match res {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to load script: {}", e);
                    continue;
                }
            };

            info!("Loaded script!");

            commands.entity(entity).insert(LoadedScript);
        }
    }
}

/// Execution level of the script, granting access to additional APIs.
/// Must be present at script instantiation.
#[derive(Component, Default, PartialEq, Eq)]
pub enum ScriptExecutionLevel {
    #[default]
    World,
    /// System-level script, highest execution level within a separate pass.
    System,
}

#[derive(Component)]
pub struct LoadedScript;

/// Script was processed to begin loading.
#[derive(Component)]
pub struct ProcessedScript;

#[derive(Default, Deref)]
pub struct ScriptMap(pub Arc<Mutex<HashMap<Entity, ScriptEnv>>>);

#[derive(Resource, Deref)]
pub struct DefaultMaterial(pub Handle<StandardMaterial>);
