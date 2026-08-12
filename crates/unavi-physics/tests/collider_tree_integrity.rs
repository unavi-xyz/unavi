//! A scene may add and clear a prim's rigid body freely. Neither the physics
//! step nor another collider's proxy may suffer for it.

use std::time::Duration;

use avian3d::{
    collider_tree::{
        ColliderTreeProxyKey,
        ColliderTreeType,
        ColliderTrees,
    },
    prelude::*,
};
use bevy::prelude::*;
use unavi_physics::PhysicsPlugin;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        TransformPlugin,
        bevy::scene::ScenePlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        PhysicsPlugin,
    ))
    .init_asset::<Mesh>()
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f32(1.0 / 60.0),
    ));
    app.finish();
    app.cleanup();
    app
}

fn step(app: &mut App, times: usize) {
    for _ in 0..times {
        app.update();
    }
}

fn proxy_owner(app: &App, entity: Entity) -> Option<Entity> {
    let key = *app
        .world()
        .entity(entity)
        .get::<ColliderTreeProxyKey>()
        .expect("collider has no proxy key");
    app.world()
        .resource::<ColliderTrees>()
        .tree_for_type(key.tree_type())
        .get_proxy(key.id())
        .map(|proxy| proxy.collider)
}

fn tree_of(app: &App, entity: Entity) -> ColliderTreeType {
    app.world()
        .entity(entity)
        .get::<ColliderTreeProxyKey>()
        .expect("collider has no proxy key")
        .tree_type()
}

fn proxy_count(app: &App, entity: Entity) -> usize {
    let trees = app.world().resource::<ColliderTrees>();
    ColliderTreeType::ALL
        .iter()
        .map(|tree_type| {
            trees
                .tree_for_type(*tree_type)
                .proxies
                .iter()
                .filter(|(_, proxy)| proxy.collider == entity)
                .count()
        })
        .sum()
}

fn assert_sound(app: &App, entity: Entity, what: &str) {
    assert_eq!(
        proxy_owner(app, entity),
        Some(entity),
        "{what}: proxy key names a proxy that is not its own"
    );
    assert_eq!(
        proxy_count(app, entity),
        1,
        "{what}: collider owns more than one proxy"
    );
}

/// A mote is a bare collider until a grab makes it dynamic, and bare again
/// when dropped.
fn spawn_mote(app: &mut App) -> Entity {
    app.world_mut()
        .spawn((Collider::sphere(0.1), Transform::from_xyz(0.0, 1.0, -1.0)))
        .id()
}

#[test]
fn clearing_a_rigid_body_leaves_the_collider_pointing_at_its_own_proxy() {
    let mut app = app();
    let mote = spawn_mote(&mut app);
    step(&mut app, 3);
    assert_sound(&app, mote, "bare collider");

    app.world_mut().entity_mut(mote).insert(RigidBody::Dynamic);
    step(&mut app, 3);
    assert_sound(&app, mote, "grabbed");
    assert_eq!(tree_of(&app, mote), ColliderTreeType::Dynamic);

    app.world_mut().entity_mut(mote).remove::<RigidBody>();
    step(&mut app, 3);
    assert_sound(&app, mote, "dropped");
    assert_eq!(
        tree_of(&app, mote),
        ColliderTreeType::Standalone,
        "a dropped mote kept a key into the dynamic tree it had left"
    );
}

#[test]
fn a_dropped_collider_does_not_alias_the_next_body_to_take_its_slot() {
    let mut app = app();
    let mote = spawn_mote(&mut app);
    step(&mut app, 3);

    app.world_mut().entity_mut(mote).insert(RigidBody::Dynamic);
    step(&mut app, 3);
    app.world_mut().entity_mut(mote).remove::<RigidBody>();
    step(&mut app, 3);

    let other = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.2),
            Transform::from_xyz(4.0, 5.0, 0.0),
        ))
        .id();
    step(&mut app, 3);

    assert_sound(&app, mote, "dropped mote");
    assert_sound(&app, other, "unrelated body");
    assert_ne!(
        app.world().entity(mote).get::<ColliderTreeProxyKey>(),
        app.world().entity(other).get::<ColliderTreeProxyKey>(),
        "two colliders share one proxy key",
    );
}

/// The panic this guards against: repeated grab/drop cycles alias a proxy,
/// then free it while a solver body's collider still names it, and
/// `update_solver_body_aabbs` indexes the freed slot.
#[test]
fn repeated_grab_and_drop_never_panics_the_physics_step() {
    let mut app = app();

    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::from_xyz(0.0, -2.0, 0.0),
    ));
    let mote = spawn_mote(&mut app);
    step(&mut app, 3);

    let mut others = Vec::new();
    for cycle in 0..6 {
        app.world_mut().entity_mut(mote).insert(RigidBody::Dynamic);
        step(&mut app, 3);
        assert_sound(&app, mote, "grabbed");

        app.world_mut().entity_mut(mote).remove::<RigidBody>();
        step(&mut app, 3);
        assert_sound(&app, mote, "dropped");

        others.push(
            app.world_mut()
                .spawn((
                    RigidBody::Dynamic,
                    Collider::sphere(0.2),
                    Transform::from_xyz(cycle as f32, 5.0, 0.0),
                ))
                .id(),
        );
        step(&mut app, 3);

        for other in &others {
            assert_sound(&app, *other, "unrelated body");
        }
    }
}

/// A drop and a re-grab in one frame alias a proxy before any system runs.
/// The repair still has to land before avian indexes the trees.
#[test]
fn a_same_frame_drop_and_regrab_never_panics_the_physics_step() {
    let mut app = app();
    let mote = spawn_mote(&mut app);
    let other = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.2),
            Transform::from_xyz(4.0, 5.0, 0.0),
        ))
        .id();
    step(&mut app, 3);

    for _ in 0..6 {
        app.world_mut().entity_mut(mote).insert(RigidBody::Dynamic);
        step(&mut app, 2);

        {
            let mut mote_mut = app.world_mut().entity_mut(mote);
            mote_mut.remove::<RigidBody>();
            mote_mut.insert(RigidBody::Dynamic);
        }
        step(&mut app, 2);

        app.world_mut().entity_mut(mote).remove::<RigidBody>();
        step(&mut app, 2);

        assert_sound(&app, mote, "mote");
        assert_sound(&app, other, "unrelated body");
    }
}

#[test]
fn ordinary_physics_is_untouched() {
    let mut app = app();
    app.world_mut().spawn((
        RigidBody::Static,
        Collider::cuboid(20.0, 1.0, 20.0),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    let falling = app
        .world_mut()
        .spawn((
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            Transform::from_xyz(0.0, 6.0, 0.0),
        ))
        .id();

    step(&mut app, 240);

    let y = app
        .world()
        .entity(falling)
        .get::<Transform>()
        .expect("transform")
        .translation
        .y;
    assert!(
        (0.5..1.5).contains(&y),
        "a body that should have landed on the ground is at y = {y}"
    );
    assert_sound(&app, falling, "landed body");
}
