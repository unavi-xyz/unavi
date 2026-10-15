//! Portals built from documents' portal configs. [`sync_portal_config`] is
//! the single place a prim's attribute payload becomes a live [`Portal`],
//! so the portal count cap in [`PortalLimits`] is enforced here.

use bevy::prelude::*;
use bevy_hsd::attributes::portal::PortalConfig;
use hsd::id::DocId;

use crate::{
    destination::Destination,
    portal::{
        Portal,
        PortalHome,
        PortalLimits,
        PortalLink,
        PortalSize,
        PortalTargetDoc,
    },
};

/// Marks a prim whose [`PortalConfig`] was refused promotion to a live
/// [`Portal`] because [`PortalLimits::max_portals`] was already reached.
///
/// Admitted once capacity frees by [`admit_refused_portals`]; cleared along
/// with the rest of a portal's state once the config disappears.
#[derive(Component)]
pub struct PortalRefused;

/// Mirrors a [`PortalConfig`] onto its prim as a portal: its size, the space
/// it opens onto, and the link it is glued by.
///
/// Refuses to open a new portal once [`PortalLimits::max_portals`] prims
/// already carry [`Portal`], marking it [`PortalRefused`] instead so
/// [`admit_refused_portals`] can open it later if capacity frees.
pub fn sync_portal_config(
    trigger: On<Insert, PortalConfig>,
    configs: Query<&PortalConfig>,
    existing: Query<(), With<Portal>>,
    limits: Res<PortalLimits>,
    mut commands: Commands,
) {
    let Ok(cfg) = configs.get(trigger.entity) else {
        return;
    };

    if !existing.contains(trigger.entity) && existing.iter().count() >= limits.max_portals {
        warn!(
            max = limits.max_portals,
            "portal limit reached; refusing to open another portal"
        );
        commands.entity(trigger.entity).insert(PortalRefused);
        return;
    }

    let mut entity = commands.entity(trigger.entity);
    entity.remove::<PortalRefused>();
    open_portal(&mut entity, cfg);
}

/// Admits prims [`PortalRefused`] earlier once the live portal count drops
/// below [`PortalLimits::max_portals`], e.g. after another portal closes.
///
/// Entities are admitted in a deterministic (but not insertion-ordered)
/// ascending-id order, so which portal wins a freed slot does not depend on
/// system scheduling.
pub fn admit_refused_portals(
    refused: Query<Entity, With<PortalRefused>>,
    configs: Query<&PortalConfig>,
    existing: Query<(), With<Portal>>,
    limits: Res<PortalLimits>,
    mut commands: Commands,
) {
    let mut open = existing.iter().count();
    if open >= limits.max_portals {
        return;
    }

    let mut candidates: Vec<Entity> = refused.iter().collect();
    candidates.sort_unstable_by_key(|e| e.to_bits());

    for entity in candidates {
        if open >= limits.max_portals {
            break;
        }
        let Ok(cfg) = configs.get(entity) else {
            continue;
        };
        let mut entity_commands = commands.entity(entity);
        entity_commands.remove::<PortalRefused>();
        open_portal(&mut entity_commands, cfg);
        open += 1;
    }
}

fn open_portal(entity: &mut EntityCommands, cfg: &PortalConfig) {
    entity.insert((
        Portal,
        PortalSize {
            width:  cfg.0.size_x as f32,
            height: cfg.0.size_y as f32,
        },
    ));

    let Some(dest) = cfg.0.destination.as_ref() else {
        entity.remove::<(PortalTargetDoc, PortalLink, PortalHome, Destination)>();
        return;
    };

    entity.insert(PortalTargetDoc(DocId(dest.space)));
    match dest.link {
        Some(link) => entity.insert(PortalLink(link)),
        None => entity.remove::<(PortalLink, PortalHome)>(),
    };
}

/// `try_remove` because the config is also removed by despawning the portal,
/// which leaves nothing to strip by the time the command runs.
pub fn clear_portal_config(trigger: On<Remove, PortalConfig>, mut commands: Commands) {
    commands.entity(trigger.entity).try_remove::<(
        Portal,
        PortalSize,
        PortalTargetDoc,
        PortalLink,
        PortalHome,
        Destination,
        PortalRefused,
    )>();
}

#[cfg(test)] mod tests;
