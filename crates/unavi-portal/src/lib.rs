use bevy::{asset::load_internal_asset, prelude::*};

use crate::material::{PORTAL_SHADER_HANDLE, PortalMaterial};

pub struct PortalPlugin;

pub mod create;
mod material;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PORTAL_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/portal.wgsl"),
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<PortalMaterial>::default());
    }
}

#[derive(Component, Default)]
#[relationship_target(relationship = PortalCamera)]
pub struct PortalCameras(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = PortalCameras)]
#[require(Transform)]
pub struct PortalCamera {
    /// Main camera for the portal camera to be transformed to match.
    tracking: Entity,
}

/// The destination that the portal will be connected to.
#[derive(Component)]
pub struct PortalDestination(Entity);

/// Marker component for entities that can teleport through portals.
#[derive(Component)]
pub struct PortalTraveler;
