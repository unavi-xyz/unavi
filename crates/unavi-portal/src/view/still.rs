//! The arrival still: six faces rendered from a one-way portal's arrival
//! pose into one atlas, which its surface shows behind the veil.

use std::f32::consts::FRAC_PI_2;

use bevy::{
    camera::{
        RenderTarget,
        Viewport,
        visibility::RenderLayers,
    },
    prelude::*,
};
use bevy_hsd::loaded::HsdLoaded;

use crate::{
    arrival::Arrival,
    destination::Destination,
    portal::{
        Portal,
        PortalState,
    },
    view::{
        camera::{
            PortalCamera,
            third_person_layers,
        },
        live::{
            copy_viewer_camera_extras,
            render_image,
            view_tonemapping,
            viewer_camera,
        },
    },
};

#[cfg(test)] mod tests;

/// Each face's camera forward and up, in the arrival's frame. `portal.wgsl`
/// selects faces by the same table.
pub const FACES: [(Vec3, Vec3); 6] = [
    (Vec3::X, Vec3::Y),
    (Vec3::NEG_X, Vec3::Y),
    (Vec3::Y, Vec3::Z),
    (Vec3::NEG_Y, Vec3::NEG_Z),
    (Vec3::Z, Vec3::Y),
    (Vec3::NEG_Z, Vec3::Y),
];
/// Face tiles per atlas row. `portal.wgsl` lays tiles out the same way.
const ATLAS_COLUMNS: u32 = 3;
const FACE_SIZE: u32 = 256;
/// Eye height above the arrival pose the still is taken from, in metres.
const EYE_HEIGHT: f32 = 1.6;
/// How long capture cameras keep rendering once the destination is loaded,
/// in seconds. Long enough for its pipelines to compile.
const CAPTURE_SECS: f32 = 1.0;

/// The atlas tile, in tiles, that face `face` of [`FACES`] renders into.
#[must_use]
pub const fn face_tile(face: u32) -> UVec2 {
    UVec2::new(face % ATLAS_COLUMNS, face / ATLAS_COLUMNS)
}

/// A portal's still, and the arrival it was taken from.
#[derive(Component)]
pub struct StillCapture {
    pub atlas: Handle<Image>,
    arrival:   Arrival,
    /// Elapsed seconds at which the atlas is complete.
    until:     f32,
}

impl StillCapture {
    #[must_use]
    pub fn is_ready(&self, now: f32) -> bool {
        now >= self.until
    }
}

/// A camera rendering one face of `portal`'s still.
#[derive(Component)]
#[relationship(relationship_target = StillCameras)]
#[require(PortalCamera)]
pub struct StillCamera {
    pub portal: Entity,
}

#[derive(Component, Default)]
#[relationship_target(relationship = StillCamera, linked_spawn)]
pub struct StillCameras(Vec<Entity>);

fn face_transform(arrival: &Arrival, face: usize) -> Transform {
    let (forward, up) = FACES[face];
    Transform::from_translation(arrival.translation + Vec3::Y * EYE_HEIGHT)
        .with_rotation(arrival.facing * Transform::IDENTITY.looking_to(forward, up).rotation)
}

/// Takes a still for every open portal with an [`Arrival`] once its
/// destination has loaded, keeps the cameras on the arrival as it moves, and
/// drops the still when the portal closes or gains a link.
///
/// A moving arrival moves the whole space with it, so the still stays valid
/// and is not retaken.
pub fn maintain_stills(
    mut portals: Query<
        (
            Entity,
            &PortalState,
            Option<&Arrival>,
            Option<&Destination>,
            Option<&mut StillCapture>,
            Option<&StillCameras>,
        ),
        With<Portal>,
    >,
    mut cameras: Query<(&mut Camera, &mut Transform), With<StillCamera>>,
    loaded: Query<(), With<HsdLoaded>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    for (portal, state, arrival, destination, capture, still_cameras) in &mut portals {
        let arrival = arrival.filter(|_| *state == PortalState::Open);
        let settled = destination.is_some_and(|d| loaded.contains(d.0));
        match (arrival, capture) {
            (Some(arrival), None) => {
                let arrival = *arrival;
                commands.queue(move |world: &mut World| spawn_capture(world, portal, arrival, now));
            }
            (Some(arrival), Some(mut capture)) => {
                let moved = capture.arrival != *arrival;
                if moved {
                    capture.arrival = *arrival;
                }
                if !settled {
                    capture.until = now + CAPTURE_SECS;
                }
                let active = !capture.is_ready(now);
                for (face, camera) in still_cameras.iter().flat_map(|c| c.iter()).enumerate() {
                    let Ok((mut camera, mut transform)) = cameras.get_mut(camera) else {
                        continue;
                    };
                    if moved {
                        *transform = face_transform(arrival, face);
                    }
                    if camera.is_active != active {
                        camera.is_active = active;
                    }
                }
            }
            (None, Some(_)) => {
                commands
                    .entity(portal)
                    .remove::<StillCapture>()
                    .despawn_related::<StillCameras>();
            }
            (None, None) => {}
        }
    }
}

/// Spawns the six face cameras into a fresh atlas. Waits for a viewer camera,
/// so the still is taken with the same exposure, HDR and layers.
fn spawn_capture(world: &mut World, portal: Entity, arrival: Arrival, now: f32) {
    if world.get_entity(portal).is_err() || world.get::<StillCapture>(portal).is_some() {
        return;
    }
    let Some(viewer) = viewer_camera(world) else {
        return;
    };
    let size = UVec2::new(ATLAS_COLUMNS, FACES.len() as u32 / ATLAS_COLUMNS) * FACE_SIZE;
    let atlas = world
        .resource_mut::<Assets<Image>>()
        .add(render_image(size, "PortalStillAtlas"));
    let tonemapping = view_tonemapping(world, viewer);
    let layers = third_person_layers(
        &world
            .get::<RenderLayers>(viewer)
            .cloned()
            .unwrap_or_default(),
    );

    for (face, order) in (0..FACES.len()).zip(-8_isize..) {
        let camera = world
            .spawn((
                StillCamera { portal },
                Camera {
                    // Distinct orders on one target, so the faces draw in a
                    // fixed sequence.
                    order,
                    viewport: Some(Viewport {
                        physical_position: face_tile(face as u32) * FACE_SIZE,
                        physical_size: UVec2::splat(FACE_SIZE),
                        ..default()
                    }),
                    ..default()
                },
                RenderTarget::Image(atlas.clone().into()),
                Camera3d::default(),
                face_transform(&arrival, face),
            ))
            .id();
        copy_viewer_camera_extras(world, camera, viewer);
        world.entity_mut(camera).insert((
            tonemapping,
            Projection::Perspective(PerspectiveProjection {
                fov: FRAC_PI_2,
                aspect_ratio: 1.0,
                near: 0.05,
                ..default()
            }),
            layers.clone(),
        ));
    }

    world.entity_mut(portal).insert(StillCapture {
        atlas,
        arrival,
        until: now + CAPTURE_SECS,
    });
}
