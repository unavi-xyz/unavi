use bevy::prelude::*;
pub struct PortalPlugin;

pub mod material;

impl Plugin for PortalPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Marker component for entities that can teleport through portals.
#[derive(Component)]
pub struct PortalTraveler;
