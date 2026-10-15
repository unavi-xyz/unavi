use std::f32::consts::PI;

use bevy::{
    prelude::*,
    transform::TransformPlugin,
};

use super::apply_crossings;
use crate::{
    body::{
        PortalBody,
        PortalLatch,
        PrevTranslation,
    },
    destination::Destination,
    portal::{
        Portal,
        PortalSize,
        PortalState,
        update_portal_frames,
    },
};

fn setup() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins(TransformPlugin).add_systems(
        PostUpdate,
        (update_portal_frames, apply_crossings)
            .chain()
            .before(TransformSystems::Propagate),
    );

    let near = Transform::IDENTITY;
    let far = Transform::from_xyz(10.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI));

    let dest = app
        .world_mut()
        .spawn((Portal, PortalState::Open, far, GlobalTransform::from(far)))
        .id();
    let source = app
        .world_mut()
        .spawn((
            Portal,
            PortalState::Open,
            PortalSize {
                width:  2.0,
                height: 2.0,
            },
            near,
            GlobalTransform::from(near),
            Destination(dest),
        ))
        .id();
    app.world_mut().entity_mut(dest).insert(Destination(source));

    (app, source, dest)
}

fn spawn_body(app: &mut App, pose: Transform) -> Entity {
    app.world_mut()
        .spawn((PortalBody, pose, GlobalTransform::from(pose)))
        .id()
}

#[test]
fn crosses_to_destination() {
    let (mut app, _, dest) = setup();
    let body = spawn_body(&mut app, Transform::from_xyz(0.0, 0.0, 0.1));
    app.update();

    app.world_mut()
        .entity_mut(body)
        .insert(Transform::from_xyz(0.0, 0.0, -0.1));
    app.update();

    let pos = app
        .world()
        .get::<Transform>(body)
        .expect("body transform")
        .translation;
    assert!(
        pos.distance(Vec3::new(10.0, 0.0, -0.1)) < 1.0e-4,
        "body teleported to {pos}"
    );
    assert!(
        app.world()
            .get::<PortalLatch>(body)
            .expect("portal latch")
            .0
    );
    let _ = dest;
}

/// Regression for the `PrevTranslation` zero-sentinel bug: a body whose
/// resting pose is exactly the world origin must seed on its first frame
/// like any other body, not forever re-seed because `(0,0,0)` looked like
/// "unset".
#[test]
fn body_resting_at_origin_still_crosses() {
    let (mut app, ..) = setup();
    let body = spawn_body(&mut app, Transform::from_xyz(0.0, 0.0, 0.0));
    app.update();
    assert_eq!(
        app.world()
            .get::<PrevTranslation>(body)
            .expect("prev translation")
            .0,
        Some(Vec3::ZERO)
    );

    app.world_mut()
        .entity_mut(body)
        .insert(Transform::from_xyz(0.0, 0.0, -0.1));
    app.update();

    let pos = app
        .world()
        .get::<Transform>(body)
        .expect("body transform")
        .translation;
    assert!(
        pos.distance(Vec3::new(10.0, 0.0, -0.1)) < 1.0e-4,
        "body starting at the origin failed to cross on its first real move, landed at {pos}"
    );
}

#[test]
fn distant_body_does_not_cross() {
    let (mut app, ..) = setup();
    let body = spawn_body(&mut app, Transform::from_xyz(50.0, 0.0, 0.1));
    app.update();

    app.world_mut()
        .entity_mut(body)
        .insert(Transform::from_xyz(50.0, 0.0, -0.1));
    app.update();

    let pos = app
        .world()
        .get::<Transform>(body)
        .expect("body transform")
        .translation;
    assert_eq!(pos, Vec3::new(50.0, 0.0, -0.1), "far body should not cross");
}
