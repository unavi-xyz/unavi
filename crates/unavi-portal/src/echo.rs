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
    math::Affine3A,
    mesh::{
        morph::{
            MeshMorphWeights,
            MorphWeights,
        },
        skinning::SkinnedMesh,
    },
    platform::collections::{
        HashMap,
        HashSet,
    },
    prelude::*,
};
use bevy_vrm::mtoon::MtoonMaterial;

use crate::{
    body::{
        EchoBody,
        EchoRadius,
        PortalBody,
    },
    clip::{
        ClippedBody,
        clip_body,
        clip_plane,
        clone_clipped_node,
        subtree,
        unclip_body,
        update_body_clip_plane,
    },
    destination::Destination,
    portal::{
        Portal,
        PortalFrame,
        PortalState,
        portal_transfer,
    },
};

/// Clone of a node in an echoed body's subtree, carrying it through the
/// portal; posed by [`maintain_echoes`] (root) or [`sync_echo_nodes`]
/// (descendants).
#[derive(Component, Clone, Copy)]
pub struct EchoNode {
    pub source: Entity,
}

/// Mirrored stand-in on the far side of a portal, spawned while its body
/// straddles the plane.
#[derive(Component, Clone, Copy)]
pub struct PortalEcho {
    pub body:   Entity,
    pub portal: Entity,
}

pub struct DesiredEcho {
    pose:  Transform,
    plane: Vec4,
}

/// How many `ChildOf` hops [`update_echo_radius`] walks looking for a
/// changed node's nearest [`EchoBody`] ancestor before giving up.
///
/// Avatar and scene hierarchies are at most a few dozen nodes deep; this
/// only guards against a malformed or unexpectedly deep graph.
const MAX_ECHO_ANCESTOR_WALK: usize = 64;

/// Recomputes [`EchoRadius`] for every body whose subtree bounds may have
/// changed this frame. See [`EchoRadius`] for the contract this keeps.
///
/// A body's meshes often attach asynchronously as grandchildren or deeper —
/// a glTF/VRM scene loads under the body, then bones and skinned meshes
/// populate — so dirtiness is detected on *any* entity whose `Aabb` or
/// `Children` changed, then walked up through `ChildOf` to the nearest
/// [`EchoBody`] ancestor (or itself, if it is one) and recomputed from
/// there. A descendant that loses its bounds by despawning, rather than by
/// the `Aabb` component being removed while the entity stays, is not
/// observed: there is nothing left to walk up from by the time the despawn
/// commits, so that body's radius lags until some other change touches its
/// subtree.
pub fn update_echo_radius(
    mut radii: Query<&mut EchoRadius>,
    echo_bodies: Query<(), With<EchoBody>>,
    added_bodies: Query<Entity, Added<EchoBody>>,
    changed: Query<Entity, Or<(Changed<Aabb>, Changed<Children>)>>,
    parents: Query<&ChildOf>,
    globals: Query<&GlobalTransform>,
    children: Query<&Children>,
    aabbs: Query<(&GlobalTransform, &Aabb)>,
    mut dirty: Local<HashSet<Entity>>,
) {
    dirty.clear();
    dirty.extend(&added_bodies);

    for entity in &changed {
        let mut node = entity;
        for _ in 0..MAX_ECHO_ANCESTOR_WALK {
            if echo_bodies.contains(node) {
                dirty.insert(node);
                break;
            }
            let Ok(parent) = parents.get(node) else {
                break;
            };
            node = parent.parent();
        }
    }

    for &body in &*dirty {
        let Ok(body_global) = globals.get(body) else {
            continue;
        };
        let radius = subtree_radius(body, body_global, &children, &aabbs);
        if let Ok(mut r) = radii.get_mut(body) {
            r.0 = radius;
        }
    }
}

/// Maintains mirrored clones of bodies overlapping a portal plane.
pub fn maintain_echoes(
    mut commands: Commands,
    bodies: Query<(Entity, &Transform, &EchoRadius), (With<EchoBody>, Without<PortalEcho>)>,
    portals: Query<
        (
            Entity,
            &GlobalTransform,
            &PortalFrame,
            &Destination,
            &PortalState,
        ),
        (With<Portal>, Without<PortalEcho>),
    >,
    destinations: Query<&GlobalTransform, Without<PortalEcho>>,
    parents: Query<&ChildOf>,
    clipped_bodies: Query<(Entity, &ClippedBody)>,
    mut echo_roots: Query<(Entity, &PortalEcho, &mut Transform), Without<PortalBody>>,
    mut desired: Local<HashMap<(Entity, Entity), DesiredEcho>>,
    mut straddles: Local<HashMap<Entity, (Entity, Vec4, f32)>>,
) {
    desired.clear();
    straddles.clear();

    for (portal, portal_transform, frame, destination, state) in &portals {
        if *state != PortalState::Open {
            continue;
        }
        let Ok(dest_transform) = destinations.get(destination.0) else {
            continue;
        };

        let transfer = portal_transfer(portal_transform, dest_transform);

        for (body, body_transform, radius) in &bodies {
            let radius = radius.0;
            if radius <= 0.0 {
                continue;
            }

            // Portals live in world space, so the body must too; a body
            // parented under an offset space anchor would otherwise echo in
            // the wrong cell. Composed fresh from the parent's propagated
            // global and the body's current local transform, not the body's
            // own (possibly one-frame-stale, e.g. just after a crossing)
            // `GlobalTransform`.
            let body_affine = world_affine(body, body_transform, &parents, &destinations);
            let body_world_pos = Vec3::from(body_affine.translation);

            // A body farther than its own radius from the portal's plane
            // cannot be straddling it; skip the exact local-space test.
            if body_world_pos.distance_squared(portal_transform.translation())
                > (frame.bounding_radius + radius).powi(2)
            {
                continue;
            }

            let local = frame.local_from_world.transform_point3(body_world_pos);
            if local.z.abs() > radius
                || local.x.abs() > frame.half_size.x + radius
                || local.y.abs() > frame.half_size.y + radius
            {
                continue;
            }

            let side = if local.z >= 0.0 { 1.0 } else { -1.0 };
            let affine = transfer * body_affine;
            let (scale, rotation, translation) = affine.to_scale_rotation_translation();

            desired.insert(
                (body, portal),
                DesiredEcho {
                    pose:  Transform {
                        translation,
                        rotation,
                        scale,
                    },
                    plane: clip_plane(dest_transform, side),
                },
            );

            let body_plane = clip_plane(portal_transform, side);
            straddles
                .entry(body)
                .and_modify(|(closest, plane, depth)| {
                    if local.z.abs() < *depth {
                        *closest = portal;
                        *plane = body_plane;
                        *depth = local.z.abs();
                    }
                })
                .or_insert((portal, body_plane, local.z.abs()));
        }
    }

    for (entity, echo, mut transform) in &mut echo_roots {
        if let Some(d) = desired.remove(&(echo.body, echo.portal)) {
            transform.set_if_neq(d.pose);
        } else {
            debug!(echo = ?entity, body = ?echo.body, "despawning echo");
            commands.entity(entity).despawn();
        }
    }

    for ((body, portal), d) in desired.drain() {
        debug!(?body, ?portal, pos = ?d.pose.translation, "spawning echo");
        commands.queue(move |world: &mut World| {
            spawn_echo_subtree(world, body, portal, d.pose, d.plane);
        });
    }

    for (body, clipped) in &clipped_bodies {
        match straddles.get(&body) {
            None => commands.queue(move |world: &mut World| unclip_body(world, body)),
            Some(&(portal, plane, _)) if plane != clipped.plane || portal != clipped.portal => {
                commands.queue(move |world: &mut World| {
                    update_body_clip_plane(world, body, portal, plane);
                });
            }
            Some(_) => {}
        }
    }
    for (&body, &(portal, plane, _)) in &straddles {
        if !clipped_bodies.contains(body) {
            commands.queue(move |world: &mut World| clip_body(world, body, portal, plane));
        }
    }
}

/// Copies source node transforms and morph weights onto echo clones, carrying
/// animation through the portal. Echo roots are posed by [`maintain_echoes`].
pub fn sync_echo_nodes(
    mut clones: Query<
        (&EchoNode, &mut Transform, Option<&mut MeshMorphWeights>),
        Without<PortalEcho>,
    >,
    sources: Query<(&Transform, Option<&MeshMorphWeights>), Without<EchoNode>>,
) {
    for (node, mut transform, weights) in &mut clones {
        let Ok((source_transform, source_weights)) = sources.get(node.source) else {
            continue;
        };
        transform.set_if_neq(*source_transform);
        if let (Some(mut weights), Some(source_weights)) = (weights, source_weights)
            && morph_weights(&weights) != morph_weights(source_weights)
        {
            weights.clone_from(source_weights);
        }
    }
}

fn morph_weights(weights: &MeshMorphWeights) -> &[f32] {
    match weights {
        MeshMorphWeights::Value { weights } => weights,
        MeshMorphWeights::Reference(_) => &[],
    }
}

/// World transform of a body from its parent's world pose and current local
/// transform, so a fresh local write is not lagged by unpropagated globals.
fn world_affine(
    body: Entity,
    local: &Transform,
    parents: &Query<&ChildOf>,
    globals: &Query<&GlobalTransform, Without<PortalEcho>>,
) -> Affine3A {
    let parent = parents
        .get(body)
        .ok()
        .and_then(|c| globals.get(c.parent()).ok())
        .map_or(Affine3A::IDENTITY, GlobalTransform::affine);
    parent * local.compute_affine()
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

fn spawn_echo_subtree(
    world: &mut World,
    body: Entity,
    portal: Entity,
    pose: Transform,
    plane: Vec4,
) {
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
        // mode like its body; view cameras render third-person layers
        // instead.
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
            // Skinned bounds follow the source skeleton, not the clone's
            // pose.
            world.entity_mut(clone).insert((skin, NoFrustumCulling));
        }

        if source == body {
            world.entity_mut(clone).insert(PortalEcho { body, portal });
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

#[cfg(test)] mod tests;
