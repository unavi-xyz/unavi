use avian3d::prelude::{
    Collider,
    Position,
    RigidBody,
    Rotation,
};
use bevy::prelude::*;
use bytemuck::{
    PodCastError,
    try_cast_slice,
};
use hsd::attributes::{
    Attribute,
    collider::ColliderAttr,
    slots,
};
use unavi_quota::limits::MAX_MESH_ELEMENTS;

use crate::{
    HsdSlots,
    attributes::{
        AttributeParser,
        ParseError,
        util::{
            compute_global_transform,
            valid_nonneg,
            valid_positive,
        },
    },
};

#[derive(Component, Debug, Clone, Copy)]
pub struct ColliderData(pub ColliderAttr);

#[derive(Component)]
pub struct HsdCollider;

pub struct ColliderParser;

impl AttributeParser for ColliderParser {
    fn key(&self) -> &'static str {
        ColliderAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        match payload {
            Some(payload) => {
                commands
                    .entity(prim)
                    .insert((ColliderData(ColliderAttr::decode(payload)?), HsdCollider));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<(ColliderData, HsdCollider, Collider, DisabledCollider)>();
            }
        }
        Ok(())
    }
}

pub fn rebuild_collider(
    changed: Query<
        (Entity, &ColliderData, Option<&HsdSlots>),
        Or<(Changed<ColliderData>, Changed<HsdSlots>)>,
    >,
    locals: Query<&Transform>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (prim, data, slots) in &changed {
        commands.entity(prim).remove::<Collider>();

        let seed = compute_global_transform(prim, &locals, &parents);
        let transform_valid = transform_is_valid(&seed);

        let collider = match data.0 {
            ColliderAttr::Sphere(r) => build_sphere(r),
            ColliderAttr::Capsule { height, radius } => build_capsule(height, radius),
            ColliderAttr::Cuboid { x, y, z } => build_cuboid(x, y, z),
            ColliderAttr::Cylinder { height, radius } => build_cylinder(height, radius),
            ColliderAttr::ConvexHull => {
                let Some(bytes) = slots.and_then(|s| s.0.get(slots::COLLIDER_VERTICES)) else {
                    continue;
                };
                build_convex_hull(bytes)
            }
            ColliderAttr::Trimesh => {
                let Some(slots) = slots else { continue };
                let (Some(vertices), Some(indices)) = (
                    slots.0.get(slots::COLLIDER_VERTICES),
                    slots.0.get(slots::COLLIDER_INDICES),
                ) else {
                    continue;
                };
                build_trimesh(vertices, indices)
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
}

// Seed avian's global `Position` / `Rotation` from the prim's transform
// chain; `Position::PLACEHOLDER` / `Rotation::PLACEHOLDER` would crash
// avian's `On<Add, Collider>` AABB observer. Callers must gate on
// `transform_is_valid` / `global_transform_is_valid` first.
fn insert_collider_with_seed(
    commands: &mut Commands,
    prim: Entity,
    collider: Collider,
    seed: &Transform,
) {
    commands.entity(prim).insert((
        collider,
        Position(seed.translation),
        Rotation(seed.rotation),
    ));
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

fn build_convex_hull(bytes: &[u8]) -> Option<Collider> {
    if !within_cap("convex hull", bytes) {
        return None;
    }
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

fn build_trimesh(vertex_bytes: &[u8], index_bytes: &[u8]) -> Option<Collider> {
    if !within_cap("trimesh vertices", vertex_bytes) || !within_cap("trimesh indices", index_bytes)
    {
        return None;
    }
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

fn cast_to_vec3(bytes: &[u8]) -> Result<Vec<Vec3>, PodCastError> {
    let raw: &[[f32; 3]] = try_cast_slice(bytes)?;
    Ok(raw.iter().map(|&[x, y, z]| Vec3::new(x, y, z)).collect())
}

/// Collider buffers arrive over document sync, so hull and trimesh
/// construction — both superlinear in point count — are only handed input
/// this side has bounded first.
fn within_cap(name: &str, bytes: &[u8]) -> bool {
    if bytes.len() > MAX_MESH_ELEMENTS {
        warn!(
            "{name}: buffer is {} bytes, over the cap of {MAX_MESH_ELEMENTS}",
            bytes.len()
        );
        return false;
    }
    true
}

#[derive(Component)]
pub struct DisabledCollider(pub Collider);

#[derive(Component)]
pub struct DisabledRigidBody(pub RigidBody);

fn is_valid_seed(scale: Vec3, rotation: Quat, translation: Vec3) -> bool {
    translation.is_finite()
        && rotation.is_finite()
        && scale.is_finite()
        && scale.x != 0.0
        && scale.y != 0.0
        && scale.z != 0.0
}

fn transform_is_valid(t: &Transform) -> bool {
    is_valid_seed(t.scale, t.rotation, t.translation)
}

fn global_transform_is_valid(t: &GlobalTransform) -> bool {
    let (s, r, tr) = t.to_scale_rotation_translation();
    is_valid_seed(s, r, tr)
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
