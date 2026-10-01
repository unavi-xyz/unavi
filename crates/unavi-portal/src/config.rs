//! Seams built from documents' portal configs.

use bevy::prelude::*;
use bevy_hsd::attributes::portal::PortalConfig;
use hsd::id::DocId;

use crate::{
    GluedTo,
    Seam,
    SeamHome,
    SeamLink,
    SeamSize,
    SeamTargetDoc,
};

/// Mirrors a [`PortalConfig`] onto its prim as a seam: its size, the space it
/// opens onto, and the link it is glued by.
pub fn sync_portal_config(
    trigger: On<Insert, PortalConfig>,
    portals: Query<&PortalConfig>,
    mut commands: Commands,
) {
    let Ok(cfg) = portals.get(trigger.entity) else {
        return;
    };

    let mut entity = commands.entity(trigger.entity);
    entity.insert((
        Seam,
        SeamSize {
            width:  cfg.0.size_x as f32,
            height: cfg.0.size_y as f32,
        },
    ));

    let Some(dest) = cfg.0.destination.as_ref() else {
        entity.remove::<(SeamTargetDoc, SeamLink, SeamHome, GluedTo)>();
        return;
    };

    entity.insert(SeamTargetDoc(DocId(dest.space)));
    match dest.link {
        Some(link) => entity.insert(SeamLink(link)),
        None => entity.remove::<(SeamLink, SeamHome)>(),
    };
}

/// `try_remove` because the config is also removed by despawning the portal,
/// which leaves nothing to strip by the time the command runs.
pub fn clear_portal_config(trigger: On<Remove, PortalConfig>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .try_remove::<(Seam, SeamSize, SeamTargetDoc, SeamLink, SeamHome, GluedTo)>();
}
