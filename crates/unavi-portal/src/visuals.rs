use bevy::{
    camera::{
        Exposure,
        RenderTarget,
        visibility::RenderLayers,
    },
    core_pipeline::tonemapping::{
        DebandDither,
        Tonemapping,
    },
    pbr::{
        Atmosphere,
        AtmosphereSettings,
    },
    post_process::{
        bloom::Bloom,
        dof::DepthOfField,
    },
    prelude::*,
    render::{
        render_resource::{
            Extent3d,
            TextureDescriptor,
            TextureDimension,
            TextureFormat,
            TextureUsages,
        },
        view::{
            ColorGrading,
            Hdr,
        },
    },
};
use bevy_vrm::first_person::{
    DEFAULT_RENDER_LAYERS,
    FirstPersonFlag,
};

use crate::{
    Portal,
    PortalActiveRender,
    PortalCamera,
    PortalCameras,
    PortalDestination,
    PortalSize,
    PortalState,
    TrackedCamera,
    material::{
        PortalMaterial,
        PortalParams,
    },
};

pub const PORTAL_RENDER_LAYER: usize = 5;

const CLOSED_COLOR: Color = Color::srgb(0.05, 0.05, 0.08);
const LOADING_COLOR: Color = Color::srgb(0.4, 0.4, 0.7);
const OPEN_FALLBACK_COLOR: Color = Color::srgb(0.2, 0.6, 1.0);

#[derive(Component)]
pub struct CachedSize(pub PortalSize);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct VisualKey {
    pub state:  PortalState,
    pub active: bool,
}

pub fn ensure_portal_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    portals: Query<
        (Entity, &PortalSize, Option<&CachedSize>),
        (With<Portal>, Or<(Changed<PortalSize>, Without<CachedSize>)>),
    >,
) {
    for (entity, size, cached) in &portals {
        if cached.is_some_and(|c| {
            c.0.width.to_bits() == size.width.to_bits()
                && c.0.height.to_bits() == size.height.to_bits()
        }) {
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
            CachedSize(*size),
        ));
    }
}

pub fn update_portal_state(
    mut portals: Query<(&mut PortalState, Option<&PortalDestination>), With<Portal>>,
    destinations: Query<(), With<crate::IncomingPortals>>,
) {
    for (mut state, dest) in &mut portals {
        let next = match dest {
            None => PortalState::Closed,
            Some(d) if destinations.contains(d.0) => PortalState::Open,
            Some(_) => PortalState::Loading,
        };
        if *state != next {
            *state = next;
        }
    }
}

pub fn apply_active_material(
    portals: Query<
        (Entity, &PortalState, Has<PortalActiveRender>, Option<&VisualKey>),
        With<Portal>,
    >,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (entity, state, active, key) in &portals {
        let next_key = VisualKey {
            state:  *state,
            active,
        };
        if key.is_some_and(|k| *k == next_key) {
            continue;
        }

        let want_shader = active && *state == PortalState::Open;
        if want_shader {
            commands.queue(move |world: &mut World| install_shader_visual(world, entity));
        } else {
            let color = match state {
                PortalState::Closed => CLOSED_COLOR,
                PortalState::Loading => LOADING_COLOR,
                PortalState::Open => OPEN_FALLBACK_COLOR,
            };
            let mat = std_materials.add(StandardMaterial {
                base_color: color,
                unlit: true,
                double_sided: true,
                cull_mode: None,
                ..default()
            });
            commands
                .entity(entity)
                .insert((MeshMaterial3d(mat), next_key))
                .remove::<MeshMaterial3d<PortalMaterial>>();
            commands.queue(move |world: &mut World| despawn_portal_cameras(world, entity));
        }
        commands.entity(entity).insert(next_key);
    }
}

fn despawn_portal_cameras(world: &mut World, portal: Entity) {
    let cameras: Vec<Entity> = world
        .get::<PortalCameras>(portal)
        .map(|c| c.0.clone())
        .unwrap_or_default();
    for cam in cameras {
        if let Ok(e) = world.get_entity_mut(cam) {
            e.despawn();
        }
    }
}

fn install_shader_visual(world: &mut World, portal: Entity) {
    let Some(tracked_camera) = world
        .query_filtered::<Entity, (With<Camera3d>, Without<PortalCamera>)>()
        .iter(world)
        .next()
    else {
        return;
    };

    despawn_portal_cameras(world, portal);

    let size = Extent3d {
        width: 128,
        height: 128,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            label: Some("PortalImage"),
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

    let portal_material = world.resource_mut::<Assets<PortalMaterial>>().add(PortalMaterial {
        texture:   Some(image_handle.clone()),
        cull_mode: None,
        params:    PortalParams::default(),
    });

    if let Ok(mut e) = world.get_entity_mut(portal) {
        e.insert(MeshMaterial3d(portal_material));
        e.remove::<MeshMaterial3d<StandardMaterial>>();
    }

    let camera_3d = world
        .get::<Camera3d>(tracked_camera)
        .cloned()
        .unwrap_or_default();
    let portal_camera_ent = world
        .spawn((
            PortalCamera { portal },
            TrackedCamera(tracked_camera),
            Camera {
                order: -1,
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            camera_3d,
        ))
        .id();

    copy_tracked_camera_extras(world, portal_camera_ent, tracked_camera);
}

fn copy_tracked_camera_extras(world: &mut World, portal_camera_ent: Entity, tracked_camera: Entity) {
    if let Some(v) = world.get::<Atmosphere>(tracked_camera).cloned() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<AtmosphereSettings>(tracked_camera).cloned() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Bloom>(tracked_camera).cloned() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<ColorGrading>(tracked_camera).cloned() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<DebandDither>(tracked_camera).copied() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<DepthOfField>(tracked_camera).copied() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Exposure>(tracked_camera).copied() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Hdr>(tracked_camera).copied() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Projection>(tracked_camera).cloned() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<RenderLayers>(tracked_camera).cloned() {
        let merged = v
            .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::Both])
            .without(PORTAL_RENDER_LAYER);
        world.entity_mut(portal_camera_ent).insert(merged);
    }
    if let Some(v) = world.get::<Tonemapping>(tracked_camera).copied() {
        world.entity_mut(portal_camera_ent).insert(v);
    }
}

