use bevy::prelude::*;

use crate::Script;

#[derive(EntityEvent, Clone)]
pub struct LoadLocalScript {
    pub entity: Entity,
    pub path: String,
}

pub(crate) fn load_local_script(
    trigger: On<LoadLocalScript>,
    server: Res<AssetServer>,
    mut commands: Commands,
) {
    let mut entity = commands.entity(trigger.entity);
    let name = path_to_name(&trigger.path);
    let handle = server.load(&trigger.path);
    entity.insert((Script(handle), Name::new(name)));
}

fn path_to_name(path: &str) -> String {
    let name = path.strip_prefix("wasm/").unwrap_or(path);
    let name = name.strip_suffix(".wasm").unwrap_or(name);
    name.replace('/', ":")
}
