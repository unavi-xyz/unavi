use std::f32::consts::PI;

use bevy::{
    camera::RenderTarget,
    math::Affine3A,
    prelude::*,
    render::render_resource::Extent3d,
    window::{
        PrimaryWindow,
        WindowRef,
    },
};

use crate::{
    DevelopCamera,
    GluedTo,
    Seam,
    TrackedCamera,
    material::SeamMaterial,
};

/// Resize seam image sizes when the tracked camera changes.
pub fn update_develop_image_sizes(
    mut seam_cameras: Query<(&DevelopCamera, &TrackedCamera, &mut Projection)>,
    seams: Query<&MeshMaterial3d<SeamMaterial>, With<Seam>>,
    cameras: Query<(&Camera, &RenderTarget), Without<DevelopCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut seam_materials: ResMut<Assets<SeamMaterial>>,
    manual_texture_views: Res<ManualTextureViews>,
    windows: Query<&Window, Without<PrimaryWindow>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    for (seam_camera, tracked_camera, mut projection) in &mut seam_cameras {
        let Ok((camera, render_target)) = cameras.get(tracked_camera.0) else {
            continue;
        };

        let viewport_size = camera
            .viewport
            .as_ref()
            .map_or_else(
                || match render_target {
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

        let Ok(mesh_material) = seams.get(seam_camera.seam) else {
            continue;
        };

        let Some(seam_material) = seam_materials.get(mesh_material.0.id()) else {
            continue;
        };

        let Some(texture_handle) = &seam_material.texture else {
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

        // info!(?size, "Resizing seam image");
        image.texture_descriptor.size = size;
        image.resize(size);

        // Force material to update.
        let _ = seam_materials.get_mut(mesh_material.0.id());

        projection.set_changed();
    }
}

/// Transform seam camera to match tracked camera.
pub fn update_develop_camera_transforms(
    mut seam_cameras: Query<(
        &TrackedCamera,
        &DevelopCamera,
        &mut Transform,
        &mut GlobalTransform,
    )>,
    cameras: Query<&GlobalTransform, (With<Camera>, Without<DevelopCamera>)>,
    seams: Query<(&GluedTo, &GlobalTransform), Without<DevelopCamera>>,
    destinations: Query<&GlobalTransform, Without<DevelopCamera>>,
) {
    for (tracked_camera, seam_camera, mut transform, mut global_transform) in &mut seam_cameras {
        let Ok((destination, seam_transform)) = seams.get(seam_camera.seam) else {
            continue;
        };

        let Ok(destination_transform) = destinations.get(destination.0) else {
            continue;
        };

        let Ok(camera_transform) = cameras.get(tracked_camera.0) else {
            continue;
        };

        // Mirror camera view through seam.
        let seam_to_camera = seam_transform.affine().inverse() * camera_transform.affine();
        let flipped = Affine3A::from_rotation_translation(Quat::from_rotation_y(PI), Vec3::ZERO)
            * seam_to_camera;
        let new_position = Vec3::from((destination_transform.affine() * flipped).translation);

        let camera_rot = camera_transform.rotation();
        let seam_rot = seam_transform.rotation();
        let dest_rot = destination_transform.rotation();
        let new_rotation = dest_rot * Quat::from_rotation_y(PI) * seam_rot.inverse() * camera_rot;

        let new_transform = GlobalTransform::from(Affine3A::from_rotation_translation(
            new_rotation,
            new_position,
        ));

        transform.clone_from(&new_transform.compute_transform());
        global_transform.clone_from(&new_transform);
    }
}

/// Clip the seam camera at the destination seam plane via Lengyel oblique
/// near-plane projection, so geometry between the camera and the seam cannot
/// occlude the portal view.
pub fn update_develop_camera_clip_planes(
    mut seam_cameras: Query<(&DevelopCamera, &mut Projection, &GlobalTransform)>,
    seams: Query<&GluedTo>,
    destinations: Query<&GlobalTransform, Without<DevelopCamera>>,
) {
    for (seam_camera, mut projection, transform) in &mut seam_cameras {
        let Ok(destination) = seams.get(seam_camera.seam) else {
            continue;
        };

        let Ok(destination_transform) = destinations.get(destination.0) else {
            continue;
        };

        let Projection::Perspective(perspective) = projection.as_mut() else {
            continue;
        };

        let view_from_world = transform.affine().inverse();
        let plane_point = view_from_world.transform_point3(destination_transform.translation());
        let plane_normal = view_from_world
            .transform_vector3(destination_transform.back().as_vec3())
            .normalize();

        // Orient the normal away from the camera so only the far side renders.
        let normal = if plane_normal.dot(plane_point) < 0.0 {
            -plane_normal
        } else {
            plane_normal
        };

        perspective.near_clip_plane = normal.extend(-normal.dot(plane_point));
    }
}
