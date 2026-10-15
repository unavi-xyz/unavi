use bevy::{
    camera::{
        RenderTarget,
        visibility::RenderLayers,
    },
    math::Affine3A,
    prelude::*,
    render::render_resource::Extent3d,
    window::{
        PrimaryWindow,
        WindowRef,
    },
};
use bevy_vrm::first_person::{
    DEFAULT_RENDER_LAYERS,
    FIRST_PERSON_LAYER,
    FirstPersonFlag,
};

use crate::{
    body::PortalBody,
    clip::ClippedBody,
    destination::Destination,
    portal::{
        Portal,
        portal_transfer,
    },
    render_layers::PORTAL_RENDER_LAYER,
    view::material::PortalMaterial,
};

/// Portals an RTT camera mirrors a view through. Collected onto the camera
/// it mirrors.
#[derive(Component, Default)]
#[relationship_target(relationship = PortalViewCamera)]
pub struct PortalViewCameras(Vec<Entity>);

impl PortalViewCameras {
    /// Takes the camera list without cloning it, leaving an empty one in its
    /// place; the relationship machinery updates the same as any other
    /// removal when the taken cameras are despawned.
    pub(crate) fn take(&mut self) -> Vec<Entity> {
        std::mem::take(&mut self.0)
    }
}

/// A camera rendering a live mirrored view through `portal`, composited onto
/// the portal's plane.
#[derive(Component)]
#[relationship(relationship_target = PortalViewCameras)]
#[require(Transform)]
pub struct PortalViewCamera {
    pub portal: Entity,
}

/// The camera this [`PortalViewCamera`] mirrors.
#[derive(Component)]
pub struct ViewerCamera(pub Entity);

pub fn update_view_camera_image_sizes(
    mut view_cameras: Query<(&PortalViewCamera, &ViewerCamera, &mut Projection)>,
    portals: Query<&MeshMaterial3d<PortalMaterial>, With<Portal>>,
    cameras: Query<(&Camera, &RenderTarget), Without<PortalViewCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut portal_materials: ResMut<Assets<PortalMaterial>>,
    manual_texture_views: Res<ManualTextureViews>,
    windows: Query<&Window, Without<PrimaryWindow>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    for (view_camera, viewer_camera, mut projection) in &mut view_cameras {
        let Ok((camera, render_target)) = cameras.get(viewer_camera.0) else {
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

        let Ok(mesh_material) = portals.get(view_camera.portal) else {
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

        let Some(mut image) = images.get_mut(texture_handle.id()) else {
            continue;
        };

        debug!(?size, "Resizing portal view image");
        image.texture_descriptor.size = size;
        image.resize(size);

        // Force material to update so it rebinds the resized texture.
        if let Some(material) = portal_materials.get_mut(mesh_material.0.id()) {
            material.into_inner();
        }

        projection.set_changed();
    }
}

pub fn update_view_camera_transforms(
    mut view_cameras: Query<(
        &ViewerCamera,
        &PortalViewCamera,
        &mut Transform,
        &mut GlobalTransform,
    )>,
    cameras: Query<&GlobalTransform, (With<Camera>, Without<PortalViewCamera>)>,
    portals: Query<(&Destination, &GlobalTransform), Without<PortalViewCamera>>,
    destinations: Query<&GlobalTransform, Without<PortalViewCamera>>,
) {
    for (viewer_camera, view_camera, mut transform, mut global_transform) in &mut view_cameras {
        let Ok((destination, portal_transform)) = portals.get(view_camera.portal) else {
            continue;
        };

        let Ok(destination_transform) = destinations.get(destination.0) else {
            continue;
        };

        let Ok(camera_transform) = cameras.get(viewer_camera.0) else {
            continue;
        };

        // Mirror camera view through the portal, dropping any scale picked
        // up along the way so the camera pose stays rigid.
        let mirrored =
            portal_transfer(portal_transform, destination_transform) * camera_transform.affine();
        let (_, new_rotation, new_position) = mirrored.to_scale_rotation_translation();

        let new_transform = GlobalTransform::from(Affine3A::from_rotation_translation(
            new_rotation,
            new_position,
        ));

        transform.set_if_neq(new_transform.compute_transform());
        global_transform.set_if_neq(new_transform);
    }
}

/// Clips the view camera at the destination plane via Lengyel oblique
/// near-plane projection, so geometry between camera and portal cannot
/// occlude the view.
pub fn update_view_camera_clip_planes(
    mut view_cameras: Query<(
        &PortalViewCamera,
        &mut Camera,
        &mut Projection,
        &GlobalTransform,
    )>,
    portals: Query<&Destination>,
    destinations: Query<&GlobalTransform, Without<PortalViewCamera>>,
) {
    for (view_camera, mut camera, mut projection, transform) in &mut view_cameras {
        let Ok(destination) = portals.get(view_camera.portal) else {
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

        // `camera_system` caches `clip_from_view` before propagation (last
        // frame's pose); recompute from the fresh pose so the clip
        // plane doesn't trail a frame.
        camera.computed.clip_from_view = projection.get_clip_from_view();
    }
}

/// Chooses view camera layers per frame. Bodies seen through the portal
/// render third person, except while the viewer camera's body straddles the
/// portal pair, where the view stays first person.
pub fn update_view_camera_layers(
    mut view_cameras: Query<(&PortalViewCamera, &ViewerCamera, &mut RenderLayers)>,
    viewer_layers: Query<&RenderLayers, Without<PortalViewCamera>>,
    parents: Query<&ChildOf>,
    bodies: Query<(), With<PortalBody>>,
    clipped: Query<&ClippedBody>,
    destinations: Query<&Destination>,
) {
    for (view_camera, viewer_camera, mut layers) in &mut view_cameras {
        let base = viewer_layers
            .get(viewer_camera.0)
            .cloned()
            .unwrap_or_default();

        let mut node = viewer_camera.0;
        let mut body = bodies.contains(node).then_some(node);
        while body.is_none()
            && let Ok(parent) = parents.get(node)
        {
            node = parent.parent();
            body = bodies.contains(node).then_some(node);
        }

        let straddling = body.and_then(|b| clipped.get(b).ok()).is_some_and(|c| {
            c.portal == view_camera.portal
                || destinations
                    .get(c.portal)
                    .is_ok_and(|d| d.0 == view_camera.portal)
        });

        let next = if straddling {
            base.without(PORTAL_RENDER_LAYER)
        } else {
            base.union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::ThirdPersonOnly])
                .without(FIRST_PERSON_LAYER)
                .without(PORTAL_RENDER_LAYER)
        };
        layers.set_if_neq(next);
    }
}
