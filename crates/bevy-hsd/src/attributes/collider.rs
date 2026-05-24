use avian3d::prelude::{Collider, Position, RigidBody, Rotation};
use bevy::prelude::*;
use bevy_wds::blob::{
    deps::{BlobDep, BlobDeps, BlobDepsLoaded},
    request::{BlobRequest, BlobResponse},
};
use bytemuck::{PodCastError, try_cast_slice};
use bytes::Bytes;
use hsd::{
    HSD_CONTAINER_ID,
    attributes::{Attribute, collider::ColliderAttr, hydrate_attr},
};
use loro::{ContainerID, Index, TreeID, ValueOrContainer, event::Diff};

use crate::{
    attributes::{
        ApplyEvent, AttrDataEvent, AttributeParser, DocContext, ParseError,
        util::{compute_global_transform, shallow_map_updated_keys},
    },
    diff::HsdDiffEvent,
};

#[derive(Debug)]
pub enum ColliderEvent {
    Rebuild(ColliderAttr),
}

#[derive(Component)]
pub struct HsdCollider;

pub enum ColliderBlobKind {
    ConvexHull { points: Entity },
    Trimesh { vertices: Entity, indices: Entity },
}

#[derive(Component)]
#[require(BlobDeps)]
pub struct ColliderBlobs(pub ColliderBlobKind);

#[derive(Component)]
#[relationship(relationship_target = ColliderBlobsChild)]
pub struct ColliderBlobsOwner(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ColliderBlobsOwner, linked_spawn)]
pub struct ColliderBlobsChild(Entity);

pub struct ColliderParser;

impl AttributeParser for ColliderParser {
    fn key(&self) -> &'static str {
        ColliderAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        value: Option<ValueOrContainer>,
    ) -> Result<(), ParseError> {
        if value.is_some() {
            commands.entity(prim).insert(HsdCollider);
        } else {
            commands
                .entity(prim)
                .remove::<(HsdCollider, Collider, ColliderBlobsChild)>();
        }
        Ok(())
    }

    fn parse(
        &self,
        ctx: &DocContext,
        prim: TreeID,
        path: &[(ContainerID, Index)],
        diff: Diff,
    ) -> Result<(), ParseError> {
        let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
        let meta = tree.get_meta(prim)?;

        let attr: ColliderAttr = hydrate_attr(&meta)?;

        let keys = shallow_map_updated_keys(path, diff)?;
        if keys.is_empty() {
            return Ok(());
        }

        ctx.tx
            .send(HsdDiffEvent::AttrData {
                prim,
                data: AttrDataEvent::Collider(ColliderEvent::Rebuild(attr)),
            })
            .map_err(|_| ParseError::SendDiff)?;
        Ok(())
    }
}

pub fn apply_collider(
    trigger: On<ApplyEvent<ColliderEvent>>,
    locals: Query<&Transform>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let prim = trigger.entity;
    let ColliderEvent::Rebuild(attr) = &trigger.value;

    commands.entity(prim).remove::<ColliderBlobsChild>();

    let seed = compute_global_transform(prim, &locals, &parents);
    let transform_valid = transform_is_valid(&seed);

    let collider = match attr {
        ColliderAttr::Sphere(r) => build_sphere(*r),
        ColliderAttr::Capsule { height, radius } => build_capsule(*height, *radius),
        ColliderAttr::Cuboid { x, y, z } => build_cuboid(*x, *y, *z),
        ColliderAttr::Cylinder { height, radius } => build_cylinder(*height, *radius),
        ColliderAttr::ConvexHull(hash) => {
            let child = commands.spawn(ColliderBlobsOwner(prim)).id();
            let points = commands
                .spawn((
                    BlobDep(child),
                    BlobRequest(blake3::Hash::from_bytes(hash.0)),
                ))
                .id();
            commands
                .entity(child)
                .insert(ColliderBlobs(ColliderBlobKind::ConvexHull { points }));
            return;
        }
        ColliderAttr::Trimesh { vertices, indices } => {
            let child = commands.spawn(ColliderBlobsOwner(prim)).id();
            let vertex_ent = commands
                .spawn((
                    BlobDep(child),
                    BlobRequest(blake3::Hash::from_bytes(vertices.0)),
                ))
                .id();
            let index_ent = commands
                .spawn((
                    BlobDep(child),
                    BlobRequest(blake3::Hash::from_bytes(indices.0)),
                ))
                .id();
            commands
                .entity(child)
                .insert(ColliderBlobs(ColliderBlobKind::Trimesh {
                    vertices: vertex_ent,
                    indices: index_ent,
                }));
            return;
        }
    };

    if let Some(c) = collider {
        if transform_valid {
            insert_collider_with_seed(&mut commands, prim, c, &seed);
        } else {
            commands.entity(prim).insert(DisabledCollider(c));
        }
    }
}

pub fn on_collider_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    collider_blobs: Query<(&ColliderBlobs, &ColliderBlobsOwner)>,
    mut blob_responses: Query<&mut BlobResponse>,
    locals: Query<&Transform>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let child = trigger.entity;
    let Ok((blobs, owner)) = collider_blobs.get(child) else {
        return;
    };
    let prim = owner.0;

    let collider = match &blobs.0 {
        ColliderBlobKind::ConvexHull { points } => {
            let Ok(Some(bytes)) = blob_responses.get_mut(*points).map(|mut b| b.0.take()) else {
                warn!("convex hull blob not ready");
                commands.entity(child).try_despawn();
                return;
            };
            build_convex_hull(&bytes)
        }
        ColliderBlobKind::Trimesh { vertices, indices } => {
            let Ok(Some(vb)) = blob_responses.get_mut(*vertices).map(|mut b| b.0.take()) else {
                warn!("trimesh vertex blob not ready");
                commands.entity(child).try_despawn();
                return;
            };
            let Ok(Some(ib)) = blob_responses.get_mut(*indices).map(|mut b| b.0.take()) else {
                warn!("trimesh index blob not ready");
                commands.entity(child).try_despawn();
                return;
            };
            build_trimesh(&vb, &ib)
        }
    };

    if let Some(c) = collider {
        let seed = compute_global_transform(prim, &locals, &parents);
        if transform_is_valid(&seed) {
            insert_collider_with_seed(&mut commands, prim, c, &seed);
        } else {
            commands.entity(prim).insert(DisabledCollider(c));
        }
    }

    commands.entity(child).try_despawn();
}

// Seed avian's global `Position` / `Rotation` from the prim's transform
// chain; `Position::PLACEHOLDER` / `Rotation::PLACEHOLDER` would crash
// avian's `On<Add, Collider>` AABB observer.
fn insert_collider_with_seed(
    commands: &mut Commands,
    prim: Entity,
    collider: Collider,
    seed: &Transform,
) {
    let translation = if seed.translation.is_finite() {
        seed.translation
    } else {
        Vec3::ZERO
    };
    let rotation = if seed.rotation.x.is_finite()
        && seed.rotation.y.is_finite()
        && seed.rotation.z.is_finite()
        && seed.rotation.w.is_finite()
    {
        seed.rotation
    } else {
        Quat::IDENTITY
    };
    commands
        .entity(prim)
        .insert((collider, Position(translation), Rotation(rotation)));
}

fn valid_positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

fn valid_nonneg(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}

fn build_sphere(r: f64) -> Option<Collider> {
    if !valid_positive(r) {
        warn!("collider sphere: radius must be positive (got {r})");
        return None;
    }
    Some(Collider::sphere(r as f32))
}

fn build_capsule(height: f64, radius: f64) -> Option<Collider> {
    if !valid_positive(radius) {
        warn!("collider capsule: radius must be positive (got {radius})");
        return None;
    }
    if !valid_nonneg(height) {
        warn!("collider capsule: height must be non-negative (got {height})");
        return None;
    }
    Some(Collider::capsule(radius as f32, height as f32))
}

fn build_cuboid(x: f64, y: f64, z: f64) -> Option<Collider> {
    if !valid_positive(x) || !valid_positive(y) || !valid_positive(z) {
        warn!("collider cuboid: all dimensions must be positive (got {x}, {y}, {z})");
        return None;
    }
    Some(Collider::cuboid(x as f32, y as f32, z as f32))
}

fn build_cylinder(height: f64, radius: f64) -> Option<Collider> {
    if !valid_positive(radius) {
        warn!("collider cylinder: radius must be positive (got {radius})");
        return None;
    }
    if !valid_nonneg(height) {
        warn!("collider cylinder: height must be non-negative (got {height})");
        return None;
    }
    Some(Collider::cylinder(radius as f32, height as f32))
}

fn build_convex_hull(bytes: &Bytes) -> Option<Collider> {
    let points: Vec<Vec3> = match cast_to_vec3(bytes) {
        Ok(v) => v,
        Err(err) => {
            warn!(?err, "convex hull: failed to cast point buffer");
            return None;
        }
    };
    if points.is_empty() {
        warn!("convex hull: empty point buffer");
        return None;
    }
    let c = Collider::convex_hull(points);
    if c.is_none() {
        warn!("convex hull: construction failed (degenerate points?)");
    }
    c
}

fn build_trimesh(vertex_bytes: &Bytes, index_bytes: &Bytes) -> Option<Collider> {
    let vertices: Vec<Vec3> = match cast_to_vec3(vertex_bytes) {
        Ok(v) => v,
        Err(err) => {
            warn!(?err, "trimesh: failed to cast vertex buffer");
            return None;
        }
    };
    let raw_indices: Vec<[u32; 3]> = match try_cast_slice::<u8, [u32; 3]>(index_bytes) {
        Ok(s) => s.to_vec(),
        Err(err) => {
            warn!(?err, "trimesh: failed to cast index buffer");
            return None;
        }
    };
    if vertices.is_empty() || raw_indices.is_empty() {
        warn!("trimesh: empty vertex or index buffer");
        return None;
    }
    match Collider::try_trimesh(vertices, raw_indices) {
        Ok(c) => Some(c),
        Err(err) => {
            warn!(?err, "trimesh: construction failed");
            None
        }
    }
}

fn cast_to_vec3(bytes: &Bytes) -> Result<Vec<Vec3>, PodCastError> {
    let raw: &[[f32; 3]] = try_cast_slice(bytes)?;
    Ok(raw.iter().map(|&[x, y, z]| Vec3::new(x, y, z)).collect())
}

#[derive(Component)]
pub struct DisabledCollider(pub Collider);

#[derive(Component)]
pub struct DisabledRigidBody(pub RigidBody);

fn transform_is_valid(t: &Transform) -> bool {
    t.translation.is_finite()
        && t.rotation.x.is_finite()
        && t.rotation.y.is_finite()
        && t.rotation.z.is_finite()
        && t.rotation.w.is_finite()
        && t.scale.x.is_finite()
        && t.scale.x != 0.0
        && t.scale.y.is_finite()
        && t.scale.y != 0.0
        && t.scale.z.is_finite()
        && t.scale.z != 0.0
}

fn global_transform_is_valid(t: &GlobalTransform) -> bool {
    let (s, r, tr) = t.to_scale_rotation_translation();
    s.x.is_finite()
        && s.x != 0.0
        && s.y.is_finite()
        && s.y != 0.0
        && s.z.is_finite()
        && s.z != 0.0
        && r.x.is_finite()
        && r.y.is_finite()
        && r.z.is_finite()
        && r.w.is_finite()
        && tr.x.is_finite()
        && tr.y.is_finite()
        && tr.z.is_finite()
}

pub fn watch_collider_scale(
    mut commands: Commands,
    active_col: Query<(Entity, &Collider, &GlobalTransform), Without<DisabledCollider>>,
    parked_col: Query<(Entity, &DisabledCollider, &GlobalTransform)>,
    active_rb: Query<(Entity, &RigidBody, &GlobalTransform), Without<DisabledRigidBody>>,
    parked_rb: Query<(Entity, &DisabledRigidBody, &GlobalTransform)>,
) {
    for (entity, collider, transform) in &active_col {
        if !global_transform_is_valid(transform) {
            let saved = collider.clone();
            commands
                .entity(entity)
                .remove::<Collider>()
                .insert(DisabledCollider(saved));
        }
    }
    for (entity, disabled, transform) in &parked_col {
        if global_transform_is_valid(transform) {
            let restored = disabled.0.clone();
            let seed = transform.compute_transform();
            commands.entity(entity).remove::<DisabledCollider>();
            insert_collider_with_seed(&mut commands, entity, restored, &seed);
        }
    }
    for (entity, rb, transform) in &active_rb {
        if !global_transform_is_valid(transform) {
            let saved = *rb;
            commands
                .entity(entity)
                .remove::<(RigidBody, Position, Rotation)>()
                .insert(DisabledRigidBody(saved));
        }
    }
    for (entity, disabled, transform) in &parked_rb {
        if global_transform_is_valid(transform) {
            let restored = disabled.0;
            let global = transform.compute_transform();
            commands
                .entity(entity)
                .remove::<DisabledRigidBody>()
                .insert((
                    restored,
                    Position(global.translation),
                    Rotation(global.rotation),
                ));
        }
    }
}
