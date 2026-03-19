use bevy::prelude::*;
use unavi_script::{ScriptPermissions, SpawnLocalScript, load::local::ScriptSource};

const SYSTEM_SCRIPTS: &[&str] = &["wasm/unavi/gauntlet.wasm"];

pub fn spawn_system_scripts(mut commands: Commands) {
    for &path in SYSTEM_SCRIPTS {
        let permissions = ScriptPermissions::system();

        commands.trigger(SpawnLocalScript {
            permissions,
            source: ScriptSource::Path(path.to_string()),
        });
    }
}
