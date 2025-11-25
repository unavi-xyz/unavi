use bevy::{
    camera::primitives::{Frustum, HalfSpace},
    prelude::*,
};

use crate::{IncomingPortals, PortalCamera, PortalDestination, TrackedCamera};

/// Transform portal camera to match tracked camera.
pub fn update_portal_camera_transforms(
    mut portal_cameras: Query<(
        &TrackedCamera,
        &PortalCamera,
        &mut Transform,
        &mut GlobalTransform,
    )>,
    cameras: Query<&GlobalTransform, (With<Camera>, Without<PortalCamera>)>,
    portals: Query<(&PortalDestination, &GlobalTransform), Without<PortalCamera>>,
    destinations: Query<&GlobalTransform, (With<IncomingPortals>, Without<PortalCamera>)>,
) {
    for (tracked_camera, portal_camera, mut transform, mut global_transform) in &mut portal_cameras
    {
        let Ok((destination, portal_transform)) = portals.get(portal_camera.portal) else {
            continue;
        };

        let Ok(destination_transform) = destinations.get(destination.0) else {
            continue;
        };

        let Ok(camera_transform) = cameras.get(tracked_camera.0) else {
            continue;
        };

        let new_transform = GlobalTransform::from(
            destination_transform.affine()
                * portal_transform.affine().inverse()
                * camera_transform.affine(),
        );

        transform.clone_from(&new_transform.compute_transform());
        global_transform.clone_from(&new_transform);
    }
}

/// Set portal camera near frustum to portal back.
pub fn update_portal_camera_frustums(
    mut portal_cameras: Query<(&PortalCamera, &mut Frustum, &Projection, &GlobalTransform)>,
    portals: Query<&PortalDestination>,
    destinations: Query<&GlobalTransform, (With<IncomingPortals>, Without<PortalCamera>)>,
) {
    for (portal_camera, mut frustum, projection, transform) in &mut portal_cameras {
        let Ok(destination) = portals.get(portal_camera.portal) else {
            continue;
        };

        let Ok(destination_transform) = destinations.get(destination.0) else {
            continue;
        };

        let view_projection = projection.get_clip_from_view() * transform.to_matrix().inverse();

        let mut new_frustum = Frustum::from_clip_from_world_custom_far(
            &view_projection,
            &transform.translation(),
            &transform.back(),
            projection.far(),
        );

        let half_space_normal = destination_transform.forward().to_vec3a();

        let dot = destination_transform
            .translation_vec3a()
            .dot(half_space_normal.normalize());
        let near_half_space_distance = -dot - 1e-4;

        new_frustum.half_spaces[4] =
            HalfSpace::new(half_space_normal.extend(near_half_space_distance));

        *frustum = new_frustum;
    }
}
