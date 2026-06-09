use bevy::prelude::*;
use bevy_hsd::attributes::portal::PortalConfig;
use blake3::Hash;
use loro::TreeID;
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

    entity.insert(SeamTargetDoc(Hash::from(dest.space.0)));

    match dest.receptor.as_ref().and_then(|r| {
        TreeID::try_from(r.prim.as_str())
            .ok()
            .map(|p| (Hash::from(r.document.0), p))
    }) {
        Some((document, prim)) => {
            entity.insert(SeamTargetReceptor { document, prim });
        }
        None => {
            entity.remove::<SeamTargetReceptor>();
        }
    }
}

pub fn clear_portal_config(trigger: On<Remove, PortalConfig>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .remove::<(Seam, SeamSize, SeamTargetDoc, SeamTargetReceptor, GluedTo)>();
}
