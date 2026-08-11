use avian3d::prelude::Collider;
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
use unavi_physics::{
    body::{
        DisabledCollider,
        insert_collider,
    },
    shape,
};
use unavi_quota::limits::MAX_MESH_ELEMENTS;

use crate::{
    HsdSlots,
    attributes::{
        AttributeParser,
        ParseError,
        util::compute_global_transform,
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

        let collider = match data.0 {
            ColliderAttr::Sphere(r) => shape::sphere(r as f32),
            ColliderAttr::Capsule { height, radius } => {
                shape::capsule(radius as f32, height as f32)
            }
            ColliderAttr::Cuboid { x, y, z } => shape::cuboid(x as f32, y as f32, z as f32),
            ColliderAttr::Cylinder { height, radius } => {
                shape::cylinder(radius as f32, height as f32)
            }
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
            insert_collider(&mut commands, prim, c, &seed);
        }
    }
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
