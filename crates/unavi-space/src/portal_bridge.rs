use bevy::prelude::*;
use bevy_hsd::attributes::portal::PortalConfig;
use hsd::id::DocId;
use unavi_manifold::{
    GluedTo,
    Seam,
    SeamSize,
    SeamTargetDoc,
    SeamTargetReceptor,
};

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
        entity
            .remove::<SeamTargetDoc>()
            .remove::<SeamTargetReceptor>()
            .remove::<GluedTo>();
        return;
    };

    entity.insert(SeamTargetDoc(DocId(dest.space)));

    match dest.receptor.as_ref() {
        Some(r) => {
            entity.insert(SeamTargetReceptor {
                document: r.document,
                prim:     r.prim,
            });
        }
        None => {
            entity.remove::<SeamTargetReceptor>();
        }
    }
}

/// `try_remove` because the config is also removed by despawning the portal,
/// which leaves nothing to strip by the time the command runs.
pub fn clear_portal_config(trigger: On<Remove, PortalConfig>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .try_remove::<(Seam, SeamSize, SeamTargetDoc, SeamTargetReceptor, GluedTo)>();
}
