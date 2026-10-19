//! The ring a crossing sends across the portal it passed and the one it
//! came out of.

use bevy::prelude::*;

use crate::{
    crossing::Crossed,
    portal::Portal,
};

/// The most recent crossing on a portal.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct PortalRing {
    /// Plane-local, in metres from the portal's centre.
    pub point: Vec2,
    /// Wrapped elapsed seconds, the same clock as the shader's
    /// `globals.time`.
    pub start: f32,
}

/// Starts a ring on `portal` at plane-local `point`, `start` being wrapped
/// elapsed seconds.
pub fn start_ring(commands: &mut Commands, portal: Entity, point: Vec2, start: f32) {
    commands.entity(portal).insert(PortalRing { point, start });
}

pub(crate) fn start_rings(
    event: On<Crossed>,
    portals: Query<(), With<Portal>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let start = time.elapsed_secs_wrapped();
    start_ring(&mut commands, event.portal, event.point, start);
    // The half-turn gluing two faces mirrors plane-local x.
    if portals.contains(event.destination) {
        start_ring(
            &mut commands,
            event.destination,
            Vec2::new(-event.point.x, event.point.y),
            start,
        );
    }
}
