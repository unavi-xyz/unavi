use bevy::prelude::*;
use unavi_script::{
    firewall::HsdFirewallEntities,
    load::local::{LoadLocalScript, ScriptSource},
    permissions::ScriptPermissions,
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

    let gauntlet_ent = commands
        .spawn((
            ScriptPermissions::system(),
            HsdFirewallEntities {
                read: module_ents.clone(),
                write: module_ents.clone(),
            },
        ))
        .trigger(|entity| LoadLocalScript {
            entity,
            source: ScriptSource::Path(GAUNTLET.to_string()),
        })
        .id();

    for module_ent in module_ents {
        commands.entity(module_ent).insert(HsdFirewallEntities {
            read: vec![gauntlet_ent],
            write: vec![gauntlet_ent],
        });
    }
}
