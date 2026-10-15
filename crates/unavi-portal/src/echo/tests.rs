use std::f32::consts::PI;

use bevy::{
    camera::primitives::Aabb,
    prelude::*,
    transform::TransformPlugin,
};
use bevy_vrm::mtoon::MtoonMaterial;

use crate::{
    body::{
        EchoBody,
        EchoRadius,
        PortalBody,
        PrevTranslation,
    },
    clip::{
        ClippedBody,
        ClippedMtoonMaterial,
        ClippedStandardMaterial,
    },
    crossing::apply_crossings,
    destination::Destination,
    echo::{
        EchoNode,
        PortalEcho,
        maintain_echoes,
        sync_echo_nodes,
        update_echo_radius,
    },
    portal::{
        Portal,
        PortalSize,
        PortalState,
        portal_transfer,
        update_portal_frames,
    },
};

fn setup() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((
        bevy::app::TaskPoolPlugin::default(),
        bevy::asset::AssetPlugin::default(),
        TransformPlugin,
    ))
    .init_asset::<StandardMaterial>()
    .init_asset::<ClippedStandardMaterial>()
    .init_asset::<MtoonMaterial>()
    .init_asset::<ClippedMtoonMaterial>()
    .add_systems(
        PostUpdate,
        (
            update_portal_frames,
            update_echo_radius,
            apply_crossings,
            maintain_echoes,
            sync_echo_nodes,
        )
            .chain()
            .before(TransformSystems::Propagate),
    );

    let portal_a = Transform::IDENTITY;
    let portal_b = Transform::from_xyz(10.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI));

    let dest = app
        .world_mut()
        .spawn((
            Portal,
            PortalState::Open,
            portal_b,
            GlobalTransform::from(portal_b),
        ))
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
            portal_a,
            GlobalTransform::from(portal_a),
            Destination(dest),
        ))
        .id();
    app.world_mut().entity_mut(dest).insert(Destination(source));

    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    let body_pose = Transform::from_xyz(0.0, 0.0, 0.2);
    let body = app
        .world_mut()
        .spawn((
            PortalBody,
            Mesh3d(Handle::default()),
            MeshMaterial3d(material),
            Aabb::from_min_max(Vec3::splat(-0.5), Vec3::splat(0.5)),
            body_pose,
            GlobalTransform::from(body_pose),
        ))
        .id();

    (app, body, source)
}

fn echo_material_count(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<&MeshMaterial3d<ClippedStandardMaterial>, With<PortalEcho>>()
        .iter(app.world())
        .count()
}

fn echo_pose(app: &mut App) -> Option<(Entity, GlobalTransform)> {
    app.world_mut()
        .query::<(Entity, &PortalEcho, &GlobalTransform)>()
        .iter(app.world())
        .map(|(e, _, t)| (e, *t))
        .next()
}

#[test]
fn echo_spawns_while_straddling_and_despawns_after() {
    let (mut app, body, source) = setup();
    app.update();
    app.update();

    let (echo, pose) = echo_pose(&mut app).expect("echo spawned");
    let source_tf = *app
        .world()
        .get::<GlobalTransform>(source)
        .expect("source transform");
    let dest_tf = *app
        .world()
        .get::<GlobalTransform>(
            app.world()
                .get::<Destination>(source)
                .expect("has destination")
                .0,
        )
        .expect("dest transform");
    let body_tf = *app
        .world()
        .get::<GlobalTransform>(body)
        .expect("body transform");
    let expected = portal_transfer(&source_tf, &dest_tf) * body_tf.affine();
    assert!(pose.affine().abs_diff_eq(expected, 1.0e-5));
    assert!(app.world().get::<ClippedBody>(body).is_some());

    let far = Transform::from_xyz(0.0, 0.0, 5.0);
    app.world_mut().entity_mut(body).insert(far);
    app.update();

    assert!(echo_pose(&mut app).is_none());
    assert!(app.world().get_entity(echo).is_err());
    assert!(app.world().get::<ClippedBody>(body).is_none());
}

#[test]
fn echo_tracks_body_movement() {
    let (mut app, body, _) = setup();
    app.update();
    let (echo, first) = echo_pose(&mut app).expect("echo spawned");

    app.world_mut()
        .entity_mut(body)
        .insert(Transform::from_xyz(0.3, 0.1, 0.05));
    app.update();

    let (echo_after, second) = echo_pose(&mut app).expect("echo kept");
    assert_eq!(echo, echo_after);
    assert!(first.translation().distance(second.translation()) > 0.1);
}

#[test]
fn near_side_echo_survives_crossing() {
    let (mut app, body, source) = setup();
    let dest = app.world().get::<Destination>(source).expect("glued").0;
    app.update();

    let crossing = Transform::from_xyz(0.0, 0.0, -0.05);
    app.world_mut().entity_mut(body).insert(crossing);
    app.update();

    let body_pos = app
        .world()
        .get::<Transform>(body)
        .expect("body transform")
        .translation;
    assert!(
        body_pos.distance(Vec3::new(10.0, 0.0, -0.05)) < 1.0e-4,
        "body teleported to {body_pos}"
    );

    let echoes = app
        .world_mut()
        .query::<(&PortalEcho, &GlobalTransform)>()
        .iter(app.world())
        .map(|(e, t)| (e.portal, t.translation()))
        .collect::<Vec<_>>();
    assert_eq!(echoes.len(), 1, "echoes: {echoes:?}");
    assert_eq!(echoes[0].0, dest);
    assert!(
        echoes[0].1.distance(Vec3::new(0.0, 0.0, -0.05)) < 1.0e-4,
        "echo at {}",
        echoes[0].1
    );
    assert_eq!(
        echo_material_count(&mut app),
        1,
        "near-side echo is missing its material"
    );
}

#[test]
fn echo_clones_child_meshes() {
    let (mut app, body, _) = setup();
    let child_pose = Transform::from_xyz(0.0, 0.4, 0.0);
    let child = app
        .world_mut()
        .spawn((
            Mesh3d(Handle::default()),
            Aabb::from_min_max(Vec3::splat(-0.1), Vec3::splat(0.1)),
            child_pose,
            ChildOf(body),
        ))
        .id();
    app.update();
    app.update();

    let clone = app
        .world_mut()
        .query::<(&EchoNode, &Transform, Has<PortalEcho>)>()
        .iter(app.world())
        .find(|(node, ..)| node.source == child)
        .map(|(_, t, root)| (*t, root))
        .expect("child mesh cloned");
    assert!(!clone.1);
    assert_eq!(clone.0, child_pose);
}

#[test]
fn echo_clips_mtoon_materials() {
    let (mut app, body, _) = setup();
    let mtoon = app
        .world_mut()
        .resource_mut::<Assets<MtoonMaterial>>()
        .add(MtoonMaterial::default());
    let child = app
        .world_mut()
        .spawn((
            Mesh3d(Handle::default()),
            MeshMaterial3d(mtoon),
            Aabb::from_min_max(Vec3::splat(-0.1), Vec3::splat(0.1)),
            Transform::from_xyz(0.0, 0.3, 0.0),
            ChildOf(body),
        ))
        .id();
    app.update();
    app.update();

    assert!(
        app.world()
            .get::<MeshMaterial3d<ClippedMtoonMaterial>>(child)
            .is_some(),
        "mtoon body node not clipped"
    );
    assert!(
        app.world()
            .get::<MeshMaterial3d<MtoonMaterial>>(child)
            .is_none()
    );

    let cloned = app
        .world_mut()
        .query_filtered::<&EchoNode, With<MeshMaterial3d<ClippedMtoonMaterial>>>()
        .iter(app.world())
        .any(|node| node.source == child);
    assert!(cloned, "echo mtoon node missing clipped material");
}

#[test]
fn echo_body_under_offset_anchor_echoes_in_world_space() {
    let (mut app, ..) = setup();
    let offset = Vec3::new(100.0, 0.0, 0.0);
    let anchor = app
        .world_mut()
        .spawn((
            Transform::from_translation(offset),
            GlobalTransform::from(Transform::from_translation(offset)),
        ))
        .id();

    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    // World pose (0, 0, 0.2) straddles portal A.
    let world = Vec3::new(0.0, 0.0, 0.2);
    let body = app
        .world_mut()
        .spawn((
            EchoBody,
            Mesh3d(Handle::default()),
            MeshMaterial3d(material),
            Aabb::from_min_max(Vec3::splat(-0.5), Vec3::splat(0.5)),
            Transform::from_translation(world - offset),
            GlobalTransform::from(Transform::from_translation(world)),
            ChildOf(anchor),
        ))
        .id();
    app.update();
    app.update();

    let echo = app
        .world_mut()
        .query::<(&PortalEcho, &EchoNode, &GlobalTransform)>()
        .iter(app.world())
        .find(|(_, node, _)| node.source == body)
        .map(|(_, _, t)| t.translation())
        .expect("offset body cast no echo");
    // Transfer through portal A lands the echo at world (10, 0, 0.2).
    assert!(
        echo.distance(Vec3::new(10.0, 0.0, 0.2)) < 1.0e-4,
        "echo at {echo}, expected world-space destination"
    );
}

#[test]
fn echo_body_casts_echo_but_never_crosses() {
    let (mut app, ..) = setup();
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    let pose = Transform::from_xyz(0.0, 0.0, 0.2);
    let remote = app
        .world_mut()
        .spawn((
            EchoBody,
            Mesh3d(Handle::default()),
            MeshMaterial3d(material),
            Aabb::from_min_max(Vec3::splat(-0.5), Vec3::splat(0.5)),
            pose,
            GlobalTransform::from(pose),
        ))
        .id();
    app.update();
    app.update();

    let has_echo = app
        .world_mut()
        .query::<(&PortalEcho, &EchoNode)>()
        .iter(app.world())
        .any(|(_, node)| node.source == remote);
    assert!(has_echo, "echo-only body cast no echo");

    // Crossing state is exclusive to teleporting bodies; an echo body must
    // never be dragged through a portal by the local simulation.
    assert!(app.world().get::<PrevTranslation>(remote).is_none());
    assert!(app.world().get::<ClippedBody>(remote).is_some());
}

#[test]
fn closed_portal_spawns_no_echo() {
    let (mut app, _, source) = setup();
    app.world_mut()
        .entity_mut(source)
        .insert(PortalState::Closed);
    app.update();
    assert!(echo_pose(&mut app).is_none());
}

/// Regression for bounds attaching asynchronously, as a VRM/glTF scene does:
/// a body with no mesh anywhere in its subtree yet must still pick up a
/// radius (and start echoing) once one appears several levels down.
#[test]
fn echo_radius_recomputes_when_a_grandchild_gains_an_aabb() {
    let (mut app, ..) = setup();

    let pose = Transform::from_xyz(0.0, 0.0, 0.2);
    let body = app
        .world_mut()
        .spawn((PortalBody, pose, GlobalTransform::from(pose)))
        .id();
    app.update();
    app.update();

    assert_eq!(
        app.world().get::<EchoRadius>(body).expect("echo radius").0,
        0.0,
        "a body with no Aabb anywhere in its subtree has no radius yet"
    );
    assert!(
        app.world_mut()
            .query::<(&PortalEcho, &EchoNode)>()
            .iter(app.world())
            .all(|(_, node)| node.source != body),
        "no echo should spawn for a body with zero radius"
    );

    let child = app.world_mut().spawn(ChildOf(body)).id();
    let material = app
        .world_mut()
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());
    app.world_mut().spawn((
        Mesh3d(Handle::default()),
        MeshMaterial3d(material),
        Aabb::from_min_max(Vec3::splat(-0.5), Vec3::splat(0.5)),
        Transform::IDENTITY,
        ChildOf(child),
    ));
    app.update();
    app.update();

    let radius = app.world().get::<EchoRadius>(body).expect("echo radius").0;
    assert!(
        radius > 0.0,
        "radius should recompute once a grandchild gains bounds"
    );

    let has_echo = app
        .world_mut()
        .query::<(&PortalEcho, &EchoNode)>()
        .iter(app.world())
        .any(|(_, node)| node.source == body);
    assert!(
        has_echo,
        "echo should spawn once the body gains bounds through a grandchild"
    );
}
