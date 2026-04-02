use bevy::prelude::*;
use unavi_script::{
    DocumentFirewall, LoadLocalScript, ScriptPermissions, load::local::ScriptSource,
};

const GAUNTLET: &str = "wasm/unavi/gauntlet.wasm";
const MODULES: &[&str] = &["wasm/unavi/vui_inventory.wasm", "wasm/unavi/vui_nav.wasm"];

pub fn spawn_system_scripts(mut commands: Commands) {
    let mut module_ents = Vec::new();

    for &path in MODULES {
        let ent = commands
            .spawn(ScriptPermissions::system())
            .trigger(|entity| LoadLocalScript {
                entity,
                source: ScriptSource::Path(path.to_string()),
            })
            .id();
        module_ents.push(ent);
    }

    commands
        .spawn((
            ScriptPermissions::system(),
            DocumentFirewall {
                allowed_entities: module_ents,
                ..Default::default()
            },
        ))
        .trigger(|entity| LoadLocalScript {
            entity,
            source: ScriptSource::Path(GAUNTLET.to_string()),
        });
}
