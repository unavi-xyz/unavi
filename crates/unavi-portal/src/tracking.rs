use std::f32::consts::PI;

use bevy::{
    camera::{
        RenderTarget,
        primitives::{Frustum, HalfSpace},
    },
    math::Affine3A,
    prelude::*,
    render::render_resource::Extent3d,
    window::{PrimaryWindow, WindowRef},
};

use crate::{
    IncomingPortals, Portal, PortalCamera, PortalDestination, TrackedCamera,
    material::PortalMaterial,
};

/// Resize portal image sizes when the tracked camera changes.
pub fn update_portal_image_sizes(
    mut portal_cameras: Query<(&PortalCamera, &TrackedCamera, &mut Projection)>,
    portals: Query<&MeshMaterial3d<PortalMaterial>, With<Portal>>,
    cameras: Query<&Camera, Without<PortalCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut portal_materials: ResMut<Assets<PortalMaterial>>,
    manual_texture_views: Res<ManualTextureViews>,
    windows: Query<&Window, Without<PrimaryWindow>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    for (portal_camera, tracked_camera, mut projection) in &mut portal_cameras {
        let Ok(camera) = cameras.get(tracked_camera.0) else {
            continue;
        };

        let viewport_size = camera
            .viewport
            .as_ref()
            .map_or_else(
                || match &camera.target {
                    RenderTarget::Image(image) => images.get(image.handle.id()).map(Image::size),
                    RenderTarget::None { size } => Some(*size),
                    RenderTarget::TextureView(view) => {
                        manual_texture_views.get(view).map(|v| v.size)
                    }
                    RenderTarget::Window(window) => match window {
                        WindowRef::Primary => {
                            primary_window.single().ok().map(Window::physical_size)
                        }
                        WindowRef::Entity(window_ent) => {
                            windows.get(*window_ent).ok().map(Window::physical_size)
                        }
                    },
                },
                |v| Some(v.physical_size),
            )
            .unwrap_or_else(|| UVec2::splat(128));

        let Ok(mesh_material) = portals.get(portal_camera.portal) else {
            continue;
        };

        let Some(portal_material) = portal_materials.get(mesh_material.0.id()) else {
            continue;
        };

        let Some(texture_handle) = &portal_material.texture else {
            continue;
        };

        let Some(image) = images.get(texture_handle.id()) else {
            continue;
        };

        let image_size = image.size();

        if viewport_size == image_size {
            continue;
        }

        let size = Extent3d {
            width: viewport_size.x,
            height: viewport_size.y,
            ..default()
        };

        let Some(image) = images.get_mut(texture_handle.id()) else {
            continue;
        };

        info!(?size, "Resizing portal image");
        image.texture_descriptor.size = size;
        image.resize(size);

        // Force material to update.
        let _ = portal_materials.get_mut(mesh_material.0.id());

        projection.set_changed();
    }
}

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
                * Affine3A::from_rotation_translation(Quat::from_rotation_y(PI), Vec3::ZERO)
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

        let half_space_normal = -destination_transform.forward().to_vec3a();

        let near_half_space_distance = -destination_transform
            .translation_vec3a()
            .dot(half_space_normal.normalize())
            - 1e-4;

        new_frustum.half_spaces[4] =
            HalfSpace::new(half_space_normal.extend(near_half_space_distance));

        *frustum = new_frustum;
    }
}
