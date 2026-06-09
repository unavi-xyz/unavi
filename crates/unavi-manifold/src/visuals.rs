use bevy::{
    camera::{
        Exposure,
        RenderTarget,
        visibility::RenderLayers,
    },
    pbr::{
        Atmosphere,
        AtmosphereSettings,
    },
    post_process::dof::DepthOfField,
    prelude::{
        ManualTextureViews,
        *,
    },
    render::{
        render_resource::{
            Extent3d,
            TextureDescriptor,
            TextureDimension,
            TextureFormat,
            TextureUsages,
        },
        view::Hdr,
    },
    window::{
        PrimaryWindow,
        WindowRef,
    },
};
use bevy_vrm::first_person::{
    DEFAULT_RENDER_LAYERS,
    FirstPersonFlag,
};

use crate::{
    DevelopCamera,
    DevelopCameras,
    GluedTo,
    Seam,
    SeamActiveRender,
    SeamSize,
    SeamState,
    TrackedCamera,
    material::{
        SeamMaterial,
        SeamParams,
    },
};

pub const SEAM_RENDER_LAYER: usize = 5;

const CLOSED_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
const LOADING_COLOR: Color = Color::srgb(0.7, 0.7, 0.7);
const OPEN_FALLBACK_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[derive(Component)]
pub struct CachedSize(pub SeamSize);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct VisualKey {
    pub state:  SeamState,
    pub active: bool,
}

pub fn ensure_seam_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    seams: Query<
        (Entity, &SeamSize, Option<&CachedSize>),
        (With<Seam>, Or<(Changed<SeamSize>, Without<CachedSize>)>),
    >,
) {
    for (entity, size, cached) in &seams {
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
            RenderLayers::layer(SEAM_RENDER_LAYER),
            CachedSize(*size),
        ));
    }
}

pub fn update_seam_state(
    mut seams: Query<(&mut SeamState, Option<&GluedTo>), With<Seam>>,
    incoming: Query<(), With<crate::GluedFrom>>,
    doc_roots: Query<(), With<bevy_hsd::Hsd>>,
) {
    for (mut state, dest) in &mut seams {
        let next = match dest {
            None => SeamState::Closed,
            Some(d) if incoming.contains(d.0) || doc_roots.contains(d.0) => SeamState::Open,
            Some(_) => SeamState::Loading,
        };
        if *state != next {
            *state = next;
        }
    }
}

pub fn apply_active_material(
    seams: Query<
        (
            Entity,
            &SeamState,
            Has<SeamActiveRender>,
            Option<&VisualKey>,
        ),
        With<Seam>,
    >,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    for (entity, state, active, key) in &seams {
        let next_key = VisualKey {
            state: *state,
            active,
        };
        if key.is_some_and(|k| *k == next_key) {
            continue;
        }

        let want_shader = active && *state == SeamState::Open;
        if want_shader {
            commands.queue(move |world: &mut World| install_shader_visual(world, entity));
        } else {
            let color = match state {
                SeamState::Closed => CLOSED_COLOR,
                SeamState::Loading => LOADING_COLOR,
                SeamState::Open => OPEN_FALLBACK_COLOR,
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
                .remove::<MeshMaterial3d<SeamMaterial>>();
            commands.queue(move |world: &mut World| despawn_seam_cameras(world, entity));
        }
        commands.entity(entity).insert(next_key);
    }
}

fn despawn_seam_cameras(world: &mut World, seam: Entity) {
    let cameras: Vec<Entity> = world
        .get::<DevelopCameras>(seam)
        .map(|c| c.0.clone())
        .unwrap_or_default();
    for cam in cameras {
        if let Ok(e) = world.get_entity_mut(cam) {
            e.despawn();
        }
    }
}

fn install_shader_visual(world: &mut World, seam: Entity) {
    let Some(tracked_camera) = world
        .query_filtered::<Entity, (With<Camera3d>, Without<DevelopCamera>)>()
        .iter(world)
        .next()
    else {
        return;
    };

    despawn_seam_cameras(world, seam);

    let initial_size = initial_render_size(world, tracked_camera);
    let size = Extent3d {
        width: initial_size.x,
        height: initial_size.y,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba16Float,
            label: Some("SeamImage"),
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

    let seam_material = world
        .resource_mut::<Assets<SeamMaterial>>()
        .add(SeamMaterial {
            texture:   Some(image_handle.clone()),
            cull_mode: None,
            params:    SeamParams::default(),
        });

    if let Ok(mut e) = world.get_entity_mut(seam) {
        e.insert(MeshMaterial3d(seam_material));
        e.remove::<MeshMaterial3d<StandardMaterial>>();
    }

    let camera_3d = world
        .get::<Camera3d>(tracked_camera)
        .cloned()
        .unwrap_or_default();
    let seam_camera_ent = world
        .spawn((
            DevelopCamera { seam },
            TrackedCamera(tracked_camera),
            Camera {
                order: -1,
                ..default()
            },
            RenderTarget::Image(image_handle.into()),
            camera_3d,
        ))
        .id();

    copy_tracked_camera_extras(world, seam_camera_ent, tracked_camera);
}

/// Best-effort initial viewport size for the tracked camera.
fn initial_render_size(world: &mut World, tracked_camera: Entity) -> UVec2 {
    const FALLBACK: UVec2 = UVec2::new(1024, 1024);

    let Some(camera) = world.get::<Camera>(tracked_camera) else {
        return FALLBACK;
    };

    if let Some(viewport) = camera.viewport.as_ref() {
        return viewport.physical_size;
    }

    let Some(target) = world.get::<RenderTarget>(tracked_camera) else {
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

/// Mirror the scene-stage view settings onto the seam camera. Output-stage
/// effects are deliberately omitted: the seam camera writes linear HDR
/// radiance, and the main camera applies those passes once when it renders the
/// seam mesh.
fn copy_tracked_camera_extras(world: &mut World, seam_camera_ent: Entity, tracked_camera: Entity) {
    if let Some(v) = world.get::<Atmosphere>(tracked_camera).cloned() {
        world.entity_mut(seam_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<AtmosphereSettings>(tracked_camera).cloned() {
        world.entity_mut(seam_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<DepthOfField>(tracked_camera).copied() {
        world.entity_mut(seam_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Exposure>(tracked_camera).copied() {
        world.entity_mut(seam_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Hdr>(tracked_camera).copied() {
        world.entity_mut(seam_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<Projection>(tracked_camera).cloned() {
        world.entity_mut(seam_camera_ent).insert(v);
    }
    if let Some(v) = world.get::<RenderLayers>(tracked_camera).cloned() {
        let merged = v
            .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::Both])
            .without(SEAM_RENDER_LAYER);
        world.entity_mut(seam_camera_ent).insert(merged);
    }
}
