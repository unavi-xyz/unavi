use bevy::{
    platform::collections::HashSet,
    prelude::*,
};
use bevy_hsd::attributes::portal::PortalConfig;
use iroh_docs::NamespaceId;
use unavi_policy::space::Space;

pub fn spawn_portal_space(
    trigger: On<Insert, PortalConfig>,
    portals: Query<&PortalConfig>,
    spaces: Query<&Space>,
    // A `commands.spawn` isn't visible to `spaces` until the command queue
    // flushes, so a second portal to the same destination opened the same
    // frame wouldn't see the first's spawn without this.
    mut pending: Local<HashSet<NamespaceId>>,
    mut commands: Commands,
) {
    let Ok(portal) = portals.get(trigger.entity) else {
        return;
    };
    let Some(dest) = &portal.0.destination else {
        return;
    };
    let id = NamespaceId::from(&dest.space);

    // Drop claims once `spaces` confirms them, so a destination can be
    // reclaimed after its `Space` despawns.
    pending.retain(|pending_id| !spaces.iter().any(|s| s.0 == *pending_id));

    if spaces.iter().any(|s| s.0 == id) || !pending.insert(id) {
        return;
    }

    commands.spawn(Space(id));
}
