use bevy::prelude::*;
use bevy_hsd::attributes::portal::PortalConfig;
use blake3::Hash;

use crate::Space;

// TODO ensure singularity of spaces (if multiple opens on same frame)
pub fn spawn_portal_space(
    trigger: On<Insert, PortalConfig>,
    portals: Query<&PortalConfig>,
    spaces: Query<&Space>,
    mut commands: Commands,
) {
    let Ok(portal) = portals.get(trigger.entity) else {
        return;
    };
    let Some(dest) = &portal.0.destination else {
        return;
    };
    let id = Hash::from_bytes(dest.space.0);
    if spaces.iter().any(|s| s.0 == id) {
        return;
    }
    commands.spawn(Space(id));
}
