use bevy::prelude::*;
use bevy_hsd::LoadHsdFile;
use unavi_script::permissions::ApiPermissions;

use crate::assets::assets_dir;

const GAUNTLET_HSD: &str = "hsd/unavi_gauntlet.hsd";
const MODULE_HSDS: &[&str] = &["hsd/unavi_vui_inventory.hsd", "hsd/unavi_vui_nav.hsd"];

pub fn spawn_system_scripts(mut commands: Commands) {
    let mut module_ents = Vec::new();

    for &rel_path in MODULE_HSDS {
        let path = assets_dir().join(rel_path);
        let ent = commands
            .spawn(ApiPermissions::system())
            .trigger(move |entity| LoadHsdFile { entity, path })
            .id();
        module_ents.push(ent);
    }

    let gauntlet_path = assets_dir().join(GAUNTLET_HSD);
    let gauntlet_ent = commands
        .spawn((
            ApiPermissions::system(),
            // HsdFirewallEntities {
            //     event_receive: AccessEntities::Restricted(module_ents.clone()),
            //     scene_read: AccessEntities::Restricted(module_ents.clone()),
            //     scene_write: AccessEntities::Restricted(module_ents.clone()),
            // },
        ))
        .trigger(move |entity| LoadHsdFile {
            entity,
            path: gauntlet_path,
        })
        .id();

    for module_ent in module_ents {
        commands.entity(module_ent)
        //     .insert(HsdFirewallEntities {
        //     event_receive: AccessEntities::Restricted(vec![gauntlet_ent]),
        //     scene_read: AccessEntities::Restricted(vec![gauntlet_ent]),
        //     scene_write: AccessEntities::Restricted(vec![gauntlet_ent]),
        // })
        ;
    }
}
