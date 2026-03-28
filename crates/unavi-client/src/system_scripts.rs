use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use unavi_script::{
    DocumentFirewall, HsdFirewall, ScriptPermissions, SpawnLocalScript, load::local::ScriptSource,
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

/// Give gauntlet RW access to each module's HSD doc firewall.
pub fn maintain_module_hsd_firewalls(docs: Query<(&Name, &HsdRecordId, Option<&HsdFirewall>)>) {
    let Some(gauntlet_id) = docs
        .iter()
        .find(|(n, _, _)| n.as_str() == "unavi:gauntlet")
        .map(|(_, id, _)| id.0)
    else {
        return;
    };

    for (name, _, fw_opt) in &docs {
        if !matches!(
            name.as_str(),
            "unavi:gauntlet_inventory" | "unavi:gauntlet_nav"
        ) {
            continue;
        }
        // All module doc entities get HsdFirewall from local.rs, but update anyway.
        if let Some(fw) = fw_opt {
            let mut inner = fw.0.write().expect("hsd_fw write");
            if !inner.read.contains(&gauntlet_id) {
                inner.read.push(gauntlet_id);
            }
            if !inner.write.contains(&gauntlet_id) {
                inner.write.push(gauntlet_id);
            }
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
