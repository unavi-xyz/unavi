use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    HsdDocId,
    load::LoadHsd,
};
use unavi_script::{
    firewall::{
        Access,
        Channel,
        Firewall,
    },
    permissions::ApiPermissions,
    quota::QuotaExempt,
};

const SHELL_HSD: &str = "hsd/unavi_halo.hsdz";
const TOOL_HSDS: &[&str] = &["hsd/unavi_spawner.hsdz", "hsd/unavi_physgun.hsdz"];

/// Updates the firewall with the record IDs of provided entities, once they
/// load.
#[derive(Component)]
pub struct FirewallEntities(pub HashMap<Entity, Vec<Channel>>);

pub fn spawn_system_scripts(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut tool_ents = Vec::new();

    for &path in TOOL_HSDS {
        let handle = asset_server.load(path);
        let ent = commands
            .spawn((
                LoadHsd {
                    handle,
                    on_load: None,
                },
                ApiPermissions::system(),
                QuotaExempt,
            ))
            .id();
        tool_ents.push(ent);
    }

    let shell_ent = {
        let mut fw = HashMap::new();
        for ent in &tool_ents {
            fw.insert(*ent, vec![Channel::EventRead, Channel::EventWrite]);
        }
        let handle = asset_server.load(SHELL_HSD);
        commands
            .spawn((
                LoadHsd {
                    handle,
                    on_load: None,
                },
                ApiPermissions::system(),
                QuotaExempt,
                FirewallEntities(fw),
            ))
            .id()
    };

    for tool_ent in tool_ents {
        let mut fw = HashMap::new();
        fw.insert(
            shell_ent,
            vec![
                Channel::EventRead,
                Channel::EventWrite,
                Channel::SceneRead,
                Channel::SceneWrite,
            ],
        );
        commands.entity(tool_ent).insert(FirewallEntities(fw));
    }
}

pub fn populate_firewall_entities(
    firewalls: Query<(Entity, &FirewallEntities, Option<&mut Firewall>), With<HsdDocId>>,
    ids: Query<&HsdDocId>,
    mut commands: Commands,
) {
    for (ent, fw_ents, fw) in firewalls {
        let Some(fw) = fw else {
            commands.entity(ent).insert(Firewall::open());
            continue;
        };
        let Some(mut fw) = fw.0.try_write() else {
            continue;
        };
        let mut done = true;
        for (target_ent, channels) in &fw_ents.0 {
            let Ok(id) = ids.get(*target_ent) else {
                done = false;
                continue;
            };
            for channel in channels {
                let entry = fw.entry(*channel).or_default();

                if let Access::Restricted(set) = entry {
                    set.insert(id.0);
                }
            }
        }
        if done {
            commands.entity(ent).remove::<FirewallEntities>();
        }
    }
}
