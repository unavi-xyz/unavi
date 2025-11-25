use bevy::{asset::load_internal_asset, camera::visibility::VisibilitySystems, prelude::*};

use crate::material::{PORTAL_SHADER_HANDLE, PortalMaterial};

pub struct PortalPlugin;

pub mod create;
mod material;
mod tracking;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            PORTAL_SHADER_HANDLE,
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/portal.wgsl"),
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<PortalMaterial>::default())
            .add_systems(
                PostUpdate,
                (
                    (
                        tracking::update_portal_image_sizes,
                        tracking::update_portal_camera_transforms,
                    )
                        .chain()
                        .after(TransformSystems::Propagate)
                        .before(VisibilitySystems::UpdateFrusta),
                    tracking::update_portal_camera_frustums.after(VisibilitySystems::UpdateFrusta),
                ),
            );
    }
}

#[derive(Component)]
pub struct Portal;

#[derive(Component, Default)]
#[relationship_target(relationship = PortalDestination)]
pub struct IncomingPortals(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = IncomingPortals)]
pub struct PortalDestination(Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = PortalCamera)]
pub struct PortalCameras(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = PortalCameras)]
#[require(Transform)]
pub struct PortalCamera {
    portal: Entity,
}

#[derive(Component)]
pub struct TrackedCamera(Entity);

/// Marker component for entities that can teleport through portals.
#[derive(Component)]
pub struct PortalTraveler;
