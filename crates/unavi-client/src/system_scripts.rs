use bevy::prelude::*;
use unavi_script::{
    ScriptPermissions, SpawnLocalScript, load::local::ScriptSource, permissions::ApiName,
};

const SYSTEM_SCRIPTS: &[&str] = &["wasm/unavi/vscreen.wasm"];

pub fn spawn_system_scripts(mut commands: Commands) {
    for &path in SYSTEM_SCRIPTS {
        let mut permissions = ScriptPermissions::default();
        permissions.api.insert(ApiName::LocalAgent);

        commands.trigger(SpawnLocalScript {
            permissions,
            source: ScriptSource::Path(path.to_string()),
        });
    }
}
