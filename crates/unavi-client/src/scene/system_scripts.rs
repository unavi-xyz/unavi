use bevy::prelude::*;
use bevy_hsd::instance::InstanceHsd;
use unavi_script::permissions::ApiPermissions;

const GAUNTLET_HSD: &str = "hsd/unavi_gauntlet.hsd";
const MODULE_HSDS: &[&str] = &[
    GAUNTLET_HSD,
    "hsd/unavi_vui_inventory.hsd",
    "hsd/unavi_vui_nav.hsd",
];

pub fn spawn_system_scripts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut module_ents = Vec::new();

    for &path in MODULE_HSDS {
        let handle = asset_server.load(path);
        let ent = commands
            .spawn((InstanceHsd(handle), ApiPermissions::system()))
            .id();
        module_ents.push(ent);
    }

    // let _gauntlet_ent = commands
    //     .spawn((
    //         ApiPermissions::system(),
    //         // HsdFirewallEntities {
    //         //     event_receive: AccessEntities::Restricted(module_ents.clone()),
    //         //     scene_read: AccessEntities::Restricted(module_ents.clone()),
    //         //     scene_write: AccessEntities::Restricted(module_ents.clone()),
    //         // },
    //     ))
    //     .trigger(move |entity| LoadHsdFile {
    //         entity,
    //         path: gauntlet_path,
    //     })
    //     .id();
    //
    // for module_ent in module_ents {
    //     commands.entity(module_ent)
    //     //     .insert(HsdFirewallEntities {
    //     //     event_receive: AccessEntities::Restricted(vec![gauntlet_ent]),
    //     //     scene_read: AccessEntities::Restricted(vec![gauntlet_ent]),
    //     //     scene_write: AccessEntities::Restricted(vec![gauntlet_ent]),
    //     // })
    //     ;
    // }
}
