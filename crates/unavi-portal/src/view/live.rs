//! Installing and removing a portal's live view camera, and the viewer
//! settings every portal camera mirrors.

use bevy::{
    camera::{
        Exposure,
        Hdr,
        RenderTarget,
        visibility::RenderLayers,
    },
    core_pipeline::tonemapping::Tonemapping,
    light::Atmosphere,
    pbr::AtmosphereSettings,
    post_process::dof::DepthOfField,
    prelude::*,
    render::{
        render_resource::{
            Extent3d,
            TextureDescriptor,
            TextureDimension,
            TextureFormat,
            TextureUsages,
        },
        texture::ManualTextureViews,
    },
    window::{
        PrimaryWindow,
        WindowRef,
    },
};

use crate::{
    body::PortalViewer,
    render_layers::PORTAL_RENDER_LAYER,
    view::{
        camera::{
            PortalCamera,
            PortalViewCamera,
            PortalViewCameras,
            ViewerCamera,
        },
        material::PortalMaterial,
    },
};

/// The camera portal views mirror: the [`PortalViewer`] if there is one,
/// else any scene camera.
pub(crate) fn viewer_camera(world: &mut World) -> Option<Entity> {
    world
        .query_filtered::<Entity, (With<Camera3d>, With<PortalViewer>, Without<PortalCamera>)>()
        .iter(world)
        .next()
        .or_else(|| {
            world
                .query_filtered::<Entity, (With<Camera3d>, Without<PortalCamera>)>()
                .iter(world)
                .next()
        })
}

/// A render target a portal camera draws into and the portal shader samples.
pub(crate) fn render_image(size: UVec2, label: &'static str) -> Image {
    let size = Extent3d {
        width: size.x,
        height: size.y,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            label: Some(label),
            mip_level_count: 1,
            sample_count: 1,
            size,
            usage: TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    image
}

/// Despawns `portal`'s view cameras without cloning the relationship's
/// entity list: the list is swapped out for an empty one, which the
/// relationship machinery accepts the same as any other removal.
pub(crate) fn despawn_live(world: &mut World, portal: Entity) {
    let cameras = world
        .get_mut::<PortalViewCameras>(portal)
        .map(|mut c| c.take())
        .unwrap_or_default();
    for cam in cameras {
        if let Ok(e) = world.get_entity_mut(cam) {
            e.despawn();
        }
    }
    let material = world
        .get::<MeshMaterial3d<PortalMaterial>>(portal)
        .map(|m| m.0.id());
    if let Some(id) = material
        && let Some(mut material) = world.resource_mut::<Assets<PortalMaterial>>().get_mut(id)
    {
        material.live = None;
    }
}

/// Spawns a live view camera for `portal` and points its material at the
/// camera's image. False when there is no camera to mirror yet.
pub(crate) fn install_live(world: &mut World, portal: Entity) -> bool {
    let Some(viewer_camera) = viewer_camera(world) else {
        return false;
    };
    let Some(material) = world
        .get::<MeshMaterial3d<PortalMaterial>>(portal)
        .map(|m| m.0.id())
    else {
        return false;
    };

    despawn_live(world, portal);

    let initial_size = initial_render_size(world, viewer_camera);
    let image = render_image(initial_size, "PortalViewImage");
    let image_handle = world.resource_mut::<Assets<Image>>().add(image);

    if let Some(mut material) = world
        .resource_mut::<Assets<PortalMaterial>>()
        .get_mut(material)
    {
        material.live = Some(image_handle.clone());
    }

    let camera_3d = world
        .get::<Camera3d>(viewer_camera)
        .cloned()
        .unwrap_or_default();
    let tonemapping = view_tonemapping(world, viewer_camera);
    let view_camera_ent = world
        .spawn((
            PortalViewCamera { portal },
            ViewerCamera(viewer_camera),
            Camera {
                order: -1,
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            camera_3d,
            tonemapping,
        ))
        .id();

    copy_viewer_camera_extras(world, view_camera_ent, viewer_camera);
    true
}

/// An HDR viewer camera tonemaps as a post-pass, so a portal camera's image
/// stays linear to avoid double-darkening. An LDR one tonemaps in-shader, so
/// the portal camera tonemaps the same way.
pub(crate) fn view_tonemapping(world: &World, viewer_camera: Entity) -> Tonemapping {
    if world.get::<Hdr>(viewer_camera).is_some() {
        Tonemapping::None
    } else {
        world
            .get::<Tonemapping>(viewer_camera)
            .copied()
            .unwrap_or_default()
    }
}

/// Best-effort initial viewport size for the viewer camera.
fn initial_render_size(world: &mut World, viewer_camera: Entity) -> UVec2 {
    const FALLBACK: UVec2 = UVec2::new(1024, 1024);

    let Some(camera) = world.get::<Camera>(viewer_camera) else {
        return FALLBACK;
    };

    if let Some(viewport) = camera.viewport.as_ref() {
        return viewport.physical_size;
    }

    let Some(target) = world.get::<RenderTarget>(viewer_camera) else {
        return FALLBACK;
    };
    let target = target.clone();

    match target {
        RenderTarget::Image(image) => world
            .resource::<Assets<Image>>()
            .get(image.handle.id())
            .map_or(FALLBACK, Image::size),
        RenderTarget::None { size } => size,
        RenderTarget::TextureView(view) => world
            .resource::<ManualTextureViews>()
            .get(&view)
            .map_or(FALLBACK, |v| v.size),
        RenderTarget::Window(window) => {
            let window_ent = match window {
                WindowRef::Primary => world
                    .query_filtered::<Entity, With<PrimaryWindow>>()
                    .single(world)
                    .ok(),
                WindowRef::Entity(e) => Some(e),
            };
            window_ent
                .and_then(|e| world.get::<Window>(e))
                .map_or(FALLBACK, Window::physical_size)
        }
    }
}

/// Mirrors scene-stage view settings onto a portal camera. Output-stage
/// effects are omitted; the main camera applies them once when it renders
/// the portal mesh.
pub(crate) fn copy_viewer_camera_extras(
    world: &mut World,
    view_camera_ent: Entity,
    viewer_camera: Entity,
) {
    if let Some(v) = world.get::<Atmosphere>(viewer_camera).cloned() {
        world.entity_mut(view_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<AtmosphereSettings>(viewer_camera).cloned() {
        world.entity_mut(view_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<DepthOfField>(viewer_camera).copied() {
        world.entity_mut(view_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Exposure>(viewer_camera).copied() {
        world.entity_mut(view_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Hdr>(viewer_camera).copied() {
        world.entity_mut(view_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Projection>(viewer_camera).cloned() {
        world.entity_mut(view_camera_ent).insert(v);
    }
    // Placeholder until `update_view_camera_layers` runs this frame.
    let layers = world
        .get::<RenderLayers>(viewer_camera)
        .cloned()
        .unwrap_or_default()
        .without(PORTAL_RENDER_LAYER);
    world.entity_mut(view_camera_ent).insert(layers);
}
