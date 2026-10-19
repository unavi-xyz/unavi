use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_hsd::{
    attributes::spawn::SpawnPoint,
    document::HsdDocId,
    prim::{
        Prim,
        PrimOf,
    },
};
use hsd::id::{
    DocId,
    PrimId,
};

use super::{
    Arrival,
    update_arrivals,
};
use crate::{
    destination::Destination,
    portal::Portal,
};

const SPACE: DocId = DocId([1; 32]);

fn setup() -> (App, Entity) {
    let mut app = App::new();
    app.add_systems(Update, update_arrivals);
    let root_pose = Transform::from_xyz(100.0, 0.0, 0.0);
    let root = app
        .world_mut()
        .spawn((HsdDocId(SPACE), root_pose, GlobalTransform::from(root_pose)))
        .id();
    (app, root)
}

fn spawn_point(app: &mut App, root: Entity, prim: u8, pose: Transform) -> Entity {
    app.world_mut()
        .spawn((
            SpawnPoint { radius: 0.0 },
            Prim(PrimId([prim; 16])),
            PrimOf(root),
            ChildOf(root),
            pose,
            GlobalTransform::from(pose),
        ))
        .id()
}

fn arrival_of(app: &mut App, root: Entity) -> Option<Arrival> {
    let portal = app.world_mut().spawn((Portal, Destination(root))).id();
    app.update();
    app.world().get::<Arrival>(portal).copied()
}

#[test]
fn a_one_way_portal_arrives_at_the_lowest_keyed_spawn_point() {
    let (mut app, root) = setup();
    let facing = Quat::from_rotation_y(FRAC_PI_2);
    spawn_point(&mut app, root, 9, Transform::from_xyz(5.0, 0.0, 5.0));
    spawn_point(
        &mut app,
        root,
        2,
        Transform::from_xyz(101.0, 2.0, 3.0).with_rotation(facing * Quat::from_rotation_x(0.3)),
    );

    let arrival = arrival_of(&mut app, root).expect("one-way portal has an arrival");

    assert_eq!(arrival.translation, Vec3::new(101.0, 2.0, 3.0));
    assert!(
        arrival.facing.angle_between(facing) < 1.0e-4,
        "arrival keeps the spawn point's yaw and drops its pitch"
    );
}

#[test]
fn a_destination_without_spawn_points_arrives_at_its_root() {
    let (mut app, root) = setup();
    let arrival = arrival_of(&mut app, root).expect("one-way portal has an arrival");
    assert_eq!(arrival.translation, Vec3::new(100.0, 0.0, 0.0));
}

#[test]
fn a_linked_portal_has_no_arrival() {
    let (mut app, root) = setup();
    spawn_point(&mut app, root, 1, Transform::IDENTITY);
    let far = app.world_mut().spawn(Portal).id();
    let near = app.world_mut().spawn((Portal, Destination(far))).id();
    app.update();
    assert!(app.world().get::<Arrival>(near).is_none());
}
