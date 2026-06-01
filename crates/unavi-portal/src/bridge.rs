use bevy::prelude::*;
use bevy_hsd::attributes::portal::PortalConfig;
use blake3::Hash;
use loro::TreeID;

use crate::{
    Portal,
    PortalAllowIncoming,
    PortalDestination,
    PortalSize,
    PortalTargetReceptor,
    PortalTargetSpace,
};

pub fn sync_portal_config(
    trigger: On<Insert, PortalConfig>,
    portals: Query<&PortalConfig>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok(cfg) = portals.get(entity) else {
        return;
    };
    let attr = &cfg.0;

    let mut entity_cmds = commands.entity(entity);
    entity_cmds.insert((
        Portal,
        PortalSize {
            width:  attr.size_x as f32,
            height: attr.size_y as f32,
        },
        PortalAllowIncoming(attr.allow_incoming),
    ));

    match &attr.destination {
        Some(dest) => {
            entity_cmds.insert(PortalTargetSpace(Hash::from_bytes(dest.space.0)));

            match &dest.receptor {
                Some(rcp) => {
                    if let Ok(prim_id) = TreeID::try_from(rcp.prim.as_str()) {
                        entity_cmds.insert(PortalTargetReceptor {
                            document: Hash::from_bytes(rcp.document.0),
                            prim:     prim_id,
                        });
                    } else {
                        entity_cmds.remove::<PortalTargetReceptor>();
                    }
                }
                None => {
                    entity_cmds.remove::<PortalTargetReceptor>();
                }
            }
        }
        None => {
            entity_cmds
                .remove::<PortalTargetSpace>()
                .remove::<PortalTargetReceptor>()
                .remove::<PortalDestination>();
        }
    }
}

pub fn clear_portal_config(trigger: On<Remove, PortalConfig>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .remove::<(
            Portal,
            PortalSize,
            PortalAllowIncoming,
            PortalTargetSpace,
            PortalTargetReceptor,
            PortalDestination,
        )>();
}
