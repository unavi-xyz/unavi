use bevy::prelude::*;
use unavi_script::{ScriptPermissions, SpawnLocalScript, load::local::ScriptSource};

const SYSTEM_SCRIPTS: &[&str] = &["wasm/unavi/vscreen.wasm"];

pub fn spawn_system_scripts(mut commands: Commands) {
    for &path in SYSTEM_SCRIPTS {
        commands.trigger(SpawnLocalScript {
            permissions: ScriptPermissions {
                wired_agent: true,
                wired_local_agent: true,
                ..Default::default()
            },
            source: ScriptSource::Path(path.to_string()),
        });
    }
}
