use bevy::prelude::*;
use unavi_script::{ScriptPermissions, SpawnLocalScript};

const SYSTEM_SCRIPTS: &[&str] = &["wasm/unavi/vscreen.wasm"];

pub fn spawn_system_scripts(mut commands: Commands) {
    for &path in SYSTEM_SCRIPTS {
        commands.trigger(SpawnLocalScript {
            path: path.to_string(),
            permissions: ScriptPermissions {
                wired_player: true,
                ..Default::default()
            },
        });
    }
}
