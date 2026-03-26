use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use unavi_script::{
    DocumentFirewall, ScriptPermissions, SpawnLocalScript, load::local::ScriptSource,
    permissions::ApiName,
};

const SYSTEM_SCRIPTS: &[&str] = &[
    "wasm/unavi/gauntlet_inventory.wasm",
    "wasm/unavi/gauntlet_nav.wasm",
    "wasm/unavi/gauntlet.wasm",
];

pub fn spawn_system_scripts(mut commands: Commands) {
    for &path in SYSTEM_SCRIPTS {
        let permissions = ScriptPermissions::system();

        commands.trigger(SpawnLocalScript {
            permissions,
            source: ScriptSource::Path(path.to_string()),
        });
    }
}

/// Attach an empty `DocumentFirewall` to the gauntlet doc entity when it loads.
pub fn init_gauntlet_firewall(
    mut commands: Commands,
    new_docs: Query<(Entity, &Name), Added<HsdRecordId>>,
) {
    for (ent, name) in new_docs.iter() {
        if name.as_str() == "unavi:gauntlet" {
            commands.entity(ent).insert(DocumentFirewall::default());
        }
    }
}

/// Keep the gauntlet's firewall in sync with all loaded system-permission docs.
pub fn maintain_gauntlet_firewall(
    mut firewalls: Query<(&Name, &mut DocumentFirewall)>,
    system_scripts: Query<(&HsdRecordId, &ScriptPermissions)>,
) {
    let allowed: Vec<blake3::Hash> = system_scripts
        .iter()
        .filter(|(_, p)| p.api.contains(&ApiName::System))
        .map(|(id, _)| id.0)
        .collect();

    for (name, mut fw) in &mut firewalls {
        if name.as_str() == "unavi:gauntlet" {
            fw.allowed = allowed;
            return;
        }
    }
}
