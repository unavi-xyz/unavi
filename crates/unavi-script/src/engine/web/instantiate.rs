use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use crate::{Script, load::asset::Wasm, permissions::ApiPermissions};

#[derive(Component)]
pub struct InstantiatingScript;

#[derive(Component)]
pub struct ScriptGuest;

pub fn instantiate_scripts(
    wasms: Res<Assets<Wasm>>,
    to_instantiate: Query<
        (Entity, &Script, &ApiPermissions, NameOrEntity),
        (Without<InstantiatingScript>, Without<ScriptGuest>),
    >,
    mut commands: Commands,
) {
    for (entity, script, perms, name) in to_instantiate {
        let Some(wasm) = wasms.get(&script.0) else {
            continue;
        };

        let bytes = wasm.0.clone();
        let name = name.to_string();

        spawn_async_task(async move {
            unsafe {
                crate::runtime::web::build_script(&bytes, &name).await;
            };
        });

        commands.entity(entity).insert(InstantiatingScript);
    }
}
