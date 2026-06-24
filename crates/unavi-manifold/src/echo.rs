use avian3d::prelude::{
    Collider,
    RigidBody,
};
use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{
            NoFrustumCulling,
            RenderLayers,
        },
    },
    mesh::{
        morph::{
            MeshMorphWeights,
            MorphWeights,
        },
        skinning::SkinnedMesh,
    },
    platform::collections::HashMap,
    prelude::*,
};
use bevy_vrm::mtoon::MtoonMaterial;

use crate::{
    EchoNode,
    GluedTo,
    ManifoldBody,
    Seam,
    SeamEcho,
    SeamSize,
    SeamState,
    clip::{
        ClippedBody,
        clip_body,
        clip_plane,
        clone_clipped_node,
        subtree,
        unclip_body,
        update_body_clip_plane,
    },
    seam_transfer,
};

struct DesiredEcho {
    pose:  Transform,
    plane: Vec4,
}

/// Maintains mirrored clones of bodies overlapping a seam plane. An echo is the
/// body's render subtree posed through the seam, clipped at the plane so neither
/// side protrudes; its colliders are kinematic. Runs before transform propagation.
pub fn maintain_seam_echoes(
    mut commands: Commands,
    bodies: Query<(Entity, &Transform, &GlobalTransform), (With<ManifoldBody>, Without<SeamEcho>)>,
    seams: Query<
        (Entity, &GlobalTransform, &SeamSize, &GluedTo, &SeamState),
        (With<Seam>, Without<SeamEcho>),
    >,
    destinations: Query<&GlobalTransform, Without<SeamEcho>>,
    children: Query<&Children>,
    aabbs: Query<(&GlobalTransform, &Aabb)>,
    clipped_bodies: Query<(Entity, &ClippedBody)>,
    mut echo_roots: Query<(Entity, &SeamEcho, &mut Transform), Without<ManifoldBody>>,
) {
    let mut radii: HashMap<Entity, f32> = HashMap::new();
    for (body, _, body_global) in &bodies {
        let radius = subtree_radius(body, body_global, &children, &aabbs);
        if radius > 0.0 {
            radii.insert(body, radius);
        }
    }

    let mut desired: HashMap<(Entity, Entity), DesiredEcho> = HashMap::new();
    let mut straddles: HashMap<Entity, (Entity, Vec4, f32)> = HashMap::new();

    for (seam, seam_transform, size, destination, state) in &seams {
        if *state != SeamState::Open {
            continue;
        }
        let Ok(dest_transform) = destinations.get(destination.0) else {
            continue;
        };

        let transfer = seam_transfer(seam_transform, dest_transform);
        let seam_from_world = seam_transform.affine().inverse();

        for (body, body_transform, _) in &bodies {
            let Some(&radius) = radii.get(&body) else {
                continue;
            };
            let local = seam_from_world.transform_point3(body_transform.translation);
            if local.z.abs() > radius
                || local.x.abs() > size.width / 2.0 + radius
                || local.y.abs() > size.height / 2.0 + radius
            {
                continue;
            }

            let side = if local.z >= 0.0 { 1.0 } else { -1.0 };
            let affine = transfer * body_transform.compute_affine();
            let (scale, rotation, translation) = affine.to_scale_rotation_translation();

            desired.insert(
                (body, seam),
                DesiredEcho {
                    pose:  Transform {
                        translation,
                        rotation,
                        scale,
                    },
                    plane: clip_plane(dest_transform, side),
                },
            );

            let body_plane = clip_plane(seam_transform, side);
            straddles
                .entry(body)
                .and_modify(|(closest, plane, depth)| {
                    if local.z.abs() < *depth {
                        *closest = seam;
                        *plane = body_plane;
                        *depth = local.z.abs();
                    }
                })
                .or_insert((seam, body_plane, local.z.abs()));
        }
    }

    for (entity, echo, mut transform) in &mut echo_roots {
        if let Some(d) = desired.remove(&(echo.body, echo.seam)) {
            transform.set_if_neq(d.pose);
        } else {
            debug!(echo = ?entity, body = ?echo.body, "despawning echo");
            commands.entity(entity).despawn();
        }
    }

    for ((body, seam), d) in desired {
        debug!(?body, ?seam, pos = ?d.pose.translation, "spawning echo");
        commands.queue(move |world: &mut World| {
            spawn_echo_subtree(world, body, seam, d.pose, d.plane);
        });
    }

    for (body, clipped) in &clipped_bodies {
        match straddles.get(&body) {
            None => commands.queue(move |world: &mut World| unclip_body(world, body)),
            Some(&(seam, plane, _)) if plane != clipped.plane || seam != clipped.seam => {
                commands.queue(move |world: &mut World| {
                    update_body_clip_plane(world, body, seam, plane);
                });
            }
            Some(_) => {}
        }
    }
    for (&body, &(seam, plane, _)) in &straddles {
        if !clipped_bodies.contains(body) {
            commands.queue(move |world: &mut World| clip_body(world, body, seam, plane));
        }
    }
}

/// Copies source node transforms and morph weights onto echo clones, carrying
/// animation through the seam. Echo roots are posed by [`maintain_seam_echoes`].
pub fn sync_echo_nodes(
    mut clones: Query<
        (&EchoNode, &mut Transform, Option<&mut MeshMorphWeights>),
        Without<SeamEcho>,
    >,
    sources: Query<(&Transform, Option<&MeshMorphWeights>), Without<EchoNode>>,
) {
    for (node, mut transform, weights) in &mut clones {
        let Ok((source_transform, source_weights)) = sources.get(node.source) else {
            continue;
        };
        transform.set_if_neq(*source_transform);
        if let (Some(mut weights), Some(source_weights)) = (weights, source_weights)
            && weights.weights() != source_weights.weights()
        {
            weights.clone_from(source_weights);
        }
    }
}

/// Bounding-sphere radius of a body's subtree around the body origin.
fn subtree_radius(
    body: Entity,
    body_global: &GlobalTransform,
    children: &Query<&Children>,
    aabbs: &Query<(&GlobalTransform, &Aabb)>,
) -> f32 {
    let origin = body_global.translation();
    let mut radius: f32 = 0.0;
    let mut stack = vec![body];
    while let Some(node) = stack.pop() {
        if let Ok(kids) = children.get(node) {
            stack.extend(kids.iter());
        }
        let Ok((node_global, aabb)) = aabbs.get(node) else {
            continue;
        };
        let (scale, ..) = node_global.to_scale_rotation_translation();
        let r = (Vec3::from(aabb.half_extents) * scale).length();
        let center = node_global.transform_point(Vec3::from(aabb.center));
        radius = radius.max(center.distance(origin) + r);
    }
    radius
}

fn spawn_echo_subtree(world: &mut World, body: Entity, seam: Entity, pose: Transform, plane: Vec4) {
    if world.get_entity(body).is_err() {
        return;
    }

    let sources = subtree(world, body);
    let mut map: HashMap<Entity, Entity> = HashMap::with_capacity(sources.len());
    for &source in &sources {
        let clone = world.spawn(EchoNode { source }).id();
        map.insert(source, clone);
    }

    for &source in &sources {
        let clone = map[&source];

        let transform = if source == body {
            pose
        } else {
            world.get::<Transform>(source).copied().unwrap_or_default()
        };
        world.entity_mut(clone).insert(transform);

        if let Some(v) = world.get::<Visibility>(source).copied() {
            world.entity_mut(clone).insert(v);
        }
        if let Some(v) = world.get::<Mesh3d>(source).cloned() {
            world.entity_mut(clone).insert(v);
        }
        if let Some(v) = world.get::<Aabb>(source).copied() {
            world.entity_mut(clone).insert(v);
        }
        // Layers copy verbatim: a directly viewed echo respects first-person
        // mode like its body; seam cameras render third-person layers instead.
        if let Some(v) = world.get::<RenderLayers>(source).cloned() {
            world.entity_mut(clone).insert(v);
        }
        // Morph weights must accompany a mesh with morph targets, or its bind
        // group no longer matches the specialized pipeline layout.
        if let Some(v) = world.get::<MorphWeights>(source).cloned() {
            world.entity_mut(clone).insert(v);
        }
        if let Some(v) = world.get::<MeshMorphWeights>(source).cloned() {
            world.entity_mut(clone).insert(v);
        }

        // Echo meshes are clipped at the destination plane so they do not
        // protrude out the portal's back side, like their straddling source.
        let _ = clone_clipped_node::<StandardMaterial>(world, source, clone, plane)
            || clone_clipped_node::<MtoonMaterial>(world, source, clone, plane);

        if let Some(mut skin) = world.get::<SkinnedMesh>(source).cloned() {
            for joint in &mut skin.joints {
                if let Some(&mapped) = map.get(joint) {
                    *joint = mapped;
                }
            }
            // Skinned bounds follow the source skeleton, not the clone's pose.
            world.entity_mut(clone).insert((skin, NoFrustumCulling));
        }

        if source == body {
            world.entity_mut(clone).insert(SeamEcho { body, seam });
            if let Some(collider) = world.get::<Collider>(source).cloned() {
                world
                    .entity_mut(clone)
                    .insert((collider, RigidBody::Kinematic));
            }
        } else if let Some(&parent) = world
            .get::<ChildOf>(source)
            .and_then(|c| map.get(&c.parent()))
        {
            world.entity_mut(clone).insert(ChildOf(parent));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use bevy::{
        camera::primitives::Aabb,
        prelude::*,
        transform::TransformPlugin,
    };
    use bevy_vrm::mtoon::MtoonMaterial;

    use crate::{
        EchoNode,
        GluedTo,
        ManifoldBody,
        Seam,
        SeamEcho,
        SeamSize,
        SeamState,
        clip::{
            ClippedBody,
            ClippedMtoonMaterial,
            ClippedStandardMaterial,
        },
        seam_transfer,
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
                crate::transition::apply_seam_crossings,
                super::maintain_seam_echoes,
                super::sync_echo_nodes,
            )
                .chain()
                .before(TransformSystems::Propagate),
        );

        let seam_a = Transform::IDENTITY;
        let seam_b = Transform::from_xyz(10.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(PI));

        let dest = app
            .world_mut()
            .spawn((Seam, SeamState::Open, seam_b, GlobalTransform::from(seam_b)))
            .id();
        let source = app
            .world_mut()
            .spawn((
                Seam,
                SeamState::Open,
                SeamSize {
                    width:  2.0,
                    height: 2.0,
                },
                seam_a,
                GlobalTransform::from(seam_a),
                GluedTo(dest),
            ))
            .id();
        app.world_mut().entity_mut(dest).insert(GluedTo(source));

        let material = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial::default());
        let body_pose = Transform::from_xyz(0.0, 0.0, 0.2);
        let body = app
            .world_mut()
            .spawn((
                ManifoldBody,
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
            .query_filtered::<&MeshMaterial3d<ClippedStandardMaterial>, With<SeamEcho>>()
            .iter(app.world())
            .count()
    }

    fn echo_pose(app: &mut App) -> Option<(Entity, GlobalTransform)> {
        app.world_mut()
            .query::<(Entity, &SeamEcho, &GlobalTransform)>()
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
            .get::<GlobalTransform>(app.world().get::<GluedTo>(source).expect("glued").0)
            .expect("dest transform");
        let body_tf = *app
            .world()
            .get::<GlobalTransform>(body)
            .expect("body transform");
        let expected = seam_transfer(&source_tf, &dest_tf) * body_tf.affine();
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
        let dest = app.world().get::<GluedTo>(source).expect("glued").0;
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
            .query::<(&SeamEcho, &GlobalTransform)>()
            .iter(app.world())
            .map(|(e, t)| (e.seam, t.translation()))
            .collect::<Vec<_>>();
        assert_eq!(echoes.len(), 1, "echoes: {echoes:?}");
        assert_eq!(echoes[0].0, dest);
        assert!(
            echoes[0].1.distance(Vec3::new(0.0, 0.0, -0.05)) < 1.0e-4,
            "echo at {}",
            echoes[0].1
        );
        // The body's material was swapped for a clipped variant while it
        // straddled the entry seam; the echo must still get a material.
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
            .query::<(&EchoNode, &Transform, Has<SeamEcho>)>()
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

        // The straddling body's mtoon node is swapped for a clipped variant.
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

        // Its echo clone carries a clipped mtoon material too.
        let cloned = app
            .world_mut()
            .query_filtered::<&EchoNode, With<MeshMaterial3d<ClippedMtoonMaterial>>>()
            .iter(app.world())
            .any(|node| node.source == child);
        assert!(cloned, "echo mtoon node missing clipped material");
    }

    #[test]
    fn closed_seam_spawns_no_echo() {
        let (mut app, _, source) = setup();
        app.world_mut().entity_mut(source).insert(SeamState::Closed);
        app.update();
        assert!(echo_pose(&mut app).is_none());
    }
}
