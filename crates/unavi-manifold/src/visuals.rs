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
    prelude::{
        ManualTextureViews,
        *,
    },
    render::render_resource::{
        Extent3d,
        TextureDescriptor,
        TextureDimension,
        TextureFormat,
        TextureUsages,
    },
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
    DevelopCamera,
    DevelopCameras,
    GluedTo,
    ManifoldBody,
    ManifoldViewer,
    Seam,
    SeamActiveRender,
    SeamSize,
    SeamState,
    TrackedCamera,
    clip::ClippedBody,
    material::{
        SeamMaterial,
        SeamParams,
    },
};

pub const SEAM_RENDER_LAYER: usize = 5;

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
            NoFrustumCulling,
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
        state.set_if_neq(next);
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

        // A closed seam is nothing to look at: no target, no portal plane.
        if *state == SeamState::Closed {
            commands
                .entity(entity)
                .insert((Visibility::Hidden, next_key))
                .remove::<MeshMaterial3d<SeamMaterial>>();
            commands.queue(move |world: &mut World| despawn_seam_cameras(world, entity));
            continue;
        }
        commands.entity(entity).insert(Visibility::Visible);

        let want_shader = active && *state == SeamState::Open;
        if want_shader {
            commands.queue(move |world: &mut World| install_shader_visual(world, entity, next_key));
        } else {
            let color = match state {
                SeamState::Closed => unreachable!("closed handled above"),
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

// `key` is only written once installation succeeds, so a frame without a
// usable camera retries instead of sticking on a stale visual.
fn install_shader_visual(world: &mut World, seam: Entity, key: VisualKey) {
    let mut tracked_camera = world
        .query_filtered::<Entity, (With<Camera3d>, With<ManifoldViewer>, Without<DevelopCamera>)>()
        .iter(world)
        .next();
    if tracked_camera.is_none() {
        tracked_camera = world
            .query_filtered::<Entity, (With<Camera3d>, Without<DevelopCamera>)>()
            .iter(world)
            .next();
    }
    let Some(tracked_camera) = tracked_camera else {
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

    let size = world.get::<SeamSize>(seam).copied().unwrap_or_default();
    let seam_transform = world
        .get::<GlobalTransform>(seam)
        .copied()
        .unwrap_or_default();

    let seam_material = world
        .resource_mut::<Assets<SeamMaterial>>()
        .add(SeamMaterial {
            texture:   Some(image_handle.clone()),
            cull_mode: None,
            params:    SeamParams {
                world_from_seam: seam_transform.to_matrix(),
                half_size:       Vec2::new(size.width / 2.0, size.height / 2.0),
            },
        });

    if let Ok(mut e) = world.get_entity_mut(seam) {
        e.insert((MeshMaterial3d(seam_material), key));
        e.remove::<MeshMaterial3d<StandardMaterial>>();
    }

    let camera_3d = world
        .get::<Camera3d>(tracked_camera)
        .cloned()
        .unwrap_or_default();
    // An HDR tracked camera tonemaps as a post-pass, so the RTT must stay
    // linear to avoid double-darkening; an LDR one tonemaps in-shader, so
    // tonemap here.
    let tonemapping = if world.get::<Hdr>(tracked_camera).is_some() {
        Tonemapping::None
    } else {
        world
            .get::<Tonemapping>(tracked_camera)
            .copied()
            .unwrap_or_default()
    };
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
            tonemapping,
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

/// Mirrors scene-stage view settings onto the seam camera. Output-stage effects
/// are omitted; the main camera applies them once when it renders the seam
/// mesh.
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
    // Placeholder until `update_develop_camera_layers` runs this frame.
    let layers = world
        .get::<RenderLayers>(tracked_camera)
        .cloned()
        .unwrap_or_default()
        .without(SEAM_RENDER_LAYER);
    world.entity_mut(seam_camera_ent).insert(layers);
}

/// Chooses seam camera layers per frame. Bodies seen through the portal render
/// third person, except while the tracked camera's body straddles the portal
/// pair, where the view stays first person.
pub fn update_develop_camera_layers(
    mut seam_cameras: Query<(&DevelopCamera, &TrackedCamera, &mut RenderLayers)>,
    tracked_layers: Query<&RenderLayers, Without<DevelopCamera>>,
    parents: Query<&ChildOf>,
    bodies: Query<(), With<ManifoldBody>>,
    clipped: Query<&ClippedBody>,
    glued: Query<&GluedTo>,
) {
    for (develop_camera, tracked_camera, mut layers) in &mut seam_cameras {
        let base = tracked_layers
            .get(tracked_camera.0)
            .cloned()
            .unwrap_or_default();

        let mut node = tracked_camera.0;
        let mut body = bodies.contains(node).then_some(node);
        while body.is_none()
            && let Ok(parent) = parents.get(node)
        {
            node = parent.parent();
            body = bodies.contains(node).then_some(node);
        }

        let straddling = body.and_then(|b| clipped.get(b).ok()).is_some_and(|c| {
            c.seam == develop_camera.seam
                || glued.get(c.seam).is_ok_and(|g| g.0 == develop_camera.seam)
        });

        let next = if straddling {
            base.without(SEAM_RENDER_LAYER)
        } else {
            base.union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::ThirdPersonOnly])
                .without(FIRST_PERSON_LAYER)
                .without(SEAM_RENDER_LAYER)
        };
        if *layers != next {
            *layers = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> App {
        let mut app = App::new();
        app.add_plugins((
            bevy::app::TaskPoolPlugin::default(),
            bevy::asset::AssetPlugin::default(),
            TransformPlugin,
        ))
        .init_asset::<StandardMaterial>()
        .add_systems(Update, apply_active_material);
        app
    }

    fn spawn_seam(app: &mut App, state: SeamState) -> Entity {
        app.world_mut().spawn((Seam, state)).id()
    }

    #[test]
    fn closed_seam_is_hidden() {
        let mut app = setup();
        let seam = spawn_seam(&mut app, SeamState::Closed);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(seam),
            Some(&Visibility::Hidden)
        );
    }

    #[test]
    fn opening_a_seam_makes_it_visible() {
        let mut app = setup();
        let seam = spawn_seam(&mut app, SeamState::Closed);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(seam),
            Some(&Visibility::Hidden)
        );

        app.world_mut().entity_mut(seam).insert(SeamState::Loading);
        app.update();
        assert_eq!(
            app.world().get::<Visibility>(seam),
            Some(&Visibility::Visible)
        );
    }
}
