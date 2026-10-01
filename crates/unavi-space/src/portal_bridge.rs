use bevy::prelude::*;
use bevy_hsd::{
    attributes::portal::PortalConfig,
    document::HsdDocId,
    prim::PrimOf,
};
use hsd::id::DocId;
use unavi_portal::{
    GluedTo,
    Seam,
    SeamHome,
    SeamLink,
    SeamSize,
    SeamTargetDoc,
};

use crate::view::SpaceView;

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

/// Keeps each linked seam's [`SeamHome`] on the space its document is in,
/// which is unknown until the document registers.
pub fn sync_seam_home(
    seams: Query<(Entity, &PrimOf, Option<&SeamHome>), With<SeamLink>>,
    docs: Query<&HsdDocId>,
    view: Option<Res<SpaceView>>,
    mut commands: Commands,
) {
    let Some(view) = view else {
        return;
    };
    for (seam, child, current) in &seams {
        let home = docs.get(child.0).ok().and_then(|doc| view.space_of(doc.0));
        match (home, current) {
            (Some(home), Some(cur)) if cur.0 == home => {}
            (Some(home), _) => {
                commands.entity(seam).insert(SeamHome(home));
            }
            (None, Some(_)) => {
                commands.entity(seam).remove::<SeamHome>();
            }
            (None, None) => {}
        }
    }
}

/// `try_remove` because the config is also removed by despawning the portal,
/// which leaves nothing to strip by the time the command runs.
pub fn clear_portal_config(trigger: On<Remove, PortalConfig>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .try_remove::<(Seam, SeamSize, SeamTargetDoc, SeamLink, SeamHome, GluedTo)>();
}
