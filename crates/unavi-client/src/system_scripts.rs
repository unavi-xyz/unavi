use bevy::prelude::*;
use bevy_hsd::LoadHsdFile;
use unavi_script::{firewall::HsdFirewallEntities, permissions::ScriptPermissions};

use crate::assets::assets_dir;

const GAUNTLET_HSD: &str = "hsd/gauntlet.hsd";
const MODULE_HSDS: &[&str] = &["hsd/vui_inventory.hsd", "hsd/vui_nav.hsd"];

pub fn spawn_system_scripts(mut commands: Commands) {
    let mut module_ents = Vec::new();

    for &rel_path in MODULE_HSDS {
        let path = assets_dir().join(rel_path);
        let ent = commands
            .spawn(ScriptPermissions::system())
            .trigger(move |entity| LoadHsdFile { entity, path })
            .id();
        module_ents.push(ent);
    }

    let gauntlet_path = assets_dir().join(GAUNTLET_HSD);
    let gauntlet_ent = commands
        .spawn((
            ScriptPermissions::system(),
            HsdFirewallEntities {
                read: module_ents.clone(),
                write: module_ents.clone(),
            },
        ))
        .trigger(move |entity| LoadHsdFile {
            entity,
            path: gauntlet_path,
        })
        .id();

    for module_ent in module_ents {
        commands.entity(module_ent).insert(HsdFirewallEntities {
            read: vec![gauntlet_ent],
            write: vec![gauntlet_ent],
        });
    }
}
