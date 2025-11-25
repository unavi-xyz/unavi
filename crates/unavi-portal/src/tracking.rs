use bevy::prelude::*;

use crate::{IncomingPortals, PortalCamera, PortalDestination};

pub fn update_portal_cameras(
    mut portal_cameras: Query<(&ChildOf, &PortalCamera, &mut GlobalTransform)>,
    cameras: Query<&GlobalTransform, (With<Camera>, Without<PortalCamera>)>,
    portals: Query<(&PortalDestination, &GlobalTransform), Without<PortalCamera>>,
    destinations: Query<&GlobalTransform, (With<IncomingPortals>, Without<PortalCamera>)>,
) {
    for (child_of, portal_camera, mut transform) in &mut portal_cameras {
        let portal_ent = child_of.parent();

        let Ok((destination, portal_transform)) = portals.get(portal_ent) else {
            continue;
        };

        let Ok(destination_transform) = destinations.get(destination.0) else {
            continue;
        };

        let Ok(camera_transform) = cameras.get(portal_camera.tracking) else {
            continue;
        };

        let affine_transform = destination_transform.affine()
            * portal_transform.affine().inverse()
            * camera_transform.affine();

        transform.clone_from(&GlobalTransform::from(affine_transform));
    }
}
