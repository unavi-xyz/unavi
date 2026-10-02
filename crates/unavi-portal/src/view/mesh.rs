use bevy::{
    camera::{
        Exposure,
        Hdr,
        RenderTarget,
        visibility::{
            NoFrustumCulling,
            RenderLayers,
        },
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
    destination::Destination,
    portal::{
        Portal,
        PortalSize,
        PortalState,
    },
    render_layers::PORTAL_RENDER_LAYER,
    view::{
        Viewed,
        camera::{
            PortalViewCamera,
            PortalViewCameras,
            ViewerCamera,
        },
        material::{
            PortalFallbacks,
            PortalMaterial,
            PortalParams,
        },
    },
};

#[derive(Component)]
pub struct CachedSize(pub PortalSize);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct VisualKey {
    pub state:  PortalState,
    pub active: bool,
}

pub fn ensure_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    portals: Query<
        (Entity, &PortalSize, Option<&CachedSize>),
        (With<Portal>, Or<(Changed<PortalSize>, Without<CachedSize>)>),
    >,
) {
    for (entity, size, cached) in &portals {
        if cached.is_some_and(|c| c.0 == *size) {
            continue;
        }
        let mesh = Plane3d::default()
            .mesh()
            .normal(Dir3::Z)
            .size(size.width, size.height)
            .build();
        commands.entity(entity).insert((
            Mesh3d(meshes.add(mesh)),
            RenderLayers::layer(PORTAL_RENDER_LAYER),
            NoFrustumCulling,
            CachedSize(*size),
        ));
    }
}

pub fn update_state(
    mut portals: Query<(&mut PortalState, Option<&Destination>), With<Portal>>,
    incoming: Query<(), With<crate::destination::Arrivals>>,
    doc_roots: Query<
        (),
        (
            With<bevy_hsd::document::Hsd>,
            Without<bevy_hsd::document::Unplaced>,
        ),
    >,
) {
    for (mut state, dest) in &mut portals {
        let next = match dest {
            None => PortalState::Closed,
            Some(d) if incoming.contains(d.0) || doc_roots.contains(d.0) => PortalState::Open,
            Some(_) => PortalState::Loading,
        };
        state.set_if_neq(next);
    }
}

pub fn apply_active_material(
    portals: Query<(Entity, &PortalState, Has<Viewed>, Option<&VisualKey>), With<Portal>>,
    fallbacks: Res<PortalFallbacks>,
    mut commands: Commands,
) {
    for (entity, state, active, key) in &portals {
        let next_key = VisualKey {
            state: *state,
            active,
        };
        if key.is_some_and(|k| *k == next_key) {
            continue;
        }

        // A closed portal is nothing to look at: no target, no portal
        // plane.
        if *state == PortalState::Closed {
            commands
                .entity(entity)
                .insert((Visibility::Hidden, next_key))
                .remove::<MeshMaterial3d<PortalMaterial>>();
            commands.queue(move |world: &mut World| despawn_view_cameras(world, entity));
            continue;
        }
        commands.entity(entity).insert(Visibility::Visible);

        let want_shader = active && *state == PortalState::Open;
        if want_shader {
            commands.queue(move |world: &mut World| install_shader_visual(world, entity, next_key));
        } else {
            let fallback = match state {
                PortalState::Closed => unreachable!("closed handled above"),
                PortalState::Loading => fallbacks.loading.clone(),
                PortalState::Open => fallbacks.open.clone(),
            };
            commands
                .entity(entity)
                .insert((MeshMaterial3d(fallback), next_key))
                .remove::<MeshMaterial3d<PortalMaterial>>();
            commands.queue(move |world: &mut World| despawn_view_cameras(world, entity));
        }
    }
}

/// Despawns `portal`'s view cameras without cloning the relationship's
/// entity list: the list is swapped out for an empty one, which the
/// relationship machinery accepts the same as any other removal.
fn despawn_view_cameras(world: &mut World, portal: Entity) {
    let cameras = world
        .get_mut::<PortalViewCameras>(portal)
        .map(|mut c| c.take())
        .unwrap_or_default();
    for cam in cameras {
        if let Ok(e) = world.get_entity_mut(cam) {
            e.despawn();
        }
    }
}

// `key` is only written once installation succeeds, so a frame without a
// usable camera retries instead of sticking on a stale visual.
fn install_shader_visual(world: &mut World, portal: Entity, key: VisualKey) {
    let mut viewer_camera = world
        .query_filtered::<Entity, (
            With<Camera3d>,
            With<PortalViewer>,
            Without<PortalViewCamera>,
        )>()
        .iter(world)
        .next();
    if viewer_camera.is_none() {
        viewer_camera = world
            .query_filtered::<Entity, (With<Camera3d>, Without<PortalViewCamera>)>()
            .iter(world)
            .next();
    }
    let Some(viewer_camera) = viewer_camera else {
        return;
    };

    despawn_view_cameras(world, portal);

    let initial_size = initial_render_size(world, viewer_camera);
    let size = Extent3d {
        width: initial_size.x,
        height: initial_size.y,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            label: Some("PortalViewImage"),
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
    let image_handle = world.resource_mut::<Assets<Image>>().add(image);

    let size = world.get::<PortalSize>(portal).copied().unwrap_or_default();
    let portal_transform = world
        .get::<GlobalTransform>(portal)
        .copied()
        .unwrap_or_default();

    let portal_material = world
        .resource_mut::<Assets<PortalMaterial>>()
        .add(PortalMaterial {
            texture:   Some(image_handle.clone()),
            cull_mode: None,
            params:    PortalParams {
                world_from_portal: portal_transform.to_matrix(),
                half_size:         Vec2::new(size.width / 2.0, size.height / 2.0),
            },
        });

    if let Ok(mut e) = world.get_entity_mut(portal) {
        e.insert((MeshMaterial3d(portal_material), key));
        e.remove::<MeshMaterial3d<StandardMaterial>>();
    }

    let camera_3d = world
        .get::<Camera3d>(viewer_camera)
        .cloned()
        .unwrap_or_default();
    // An HDR viewer camera tonemaps as a post-pass, so the RTT must stay
    // linear to avoid double-darkening; an LDR one tonemaps in-shader, so
    // tonemap here.
    let tonemapping = if world.get::<Hdr>(viewer_camera).is_some() {
        Tonemapping::None
    } else {
        world
            .get::<Tonemapping>(viewer_camera)
            .copied()
            .unwrap_or_default()
    };
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

/// Mirrors scene-stage view settings onto the view camera. Output-stage
/// effects are omitted; the main camera applies them once when it renders
/// the portal mesh.
fn copy_viewer_camera_extras(world: &mut World, view_camera_ent: Entity, viewer_camera: Entity) {
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

#[cfg(test)] mod tests;
