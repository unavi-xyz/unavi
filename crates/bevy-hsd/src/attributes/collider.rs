use avian3d::prelude::Collider;
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
        util::shallow_map_updated_keys,
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
                .remove::<HsdCollider>()
                .remove::<Collider>()
                .remove::<ColliderBlobs>()
                .remove::<BlobDeps>()
                .remove::<BlobDepsLoaded>();
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
    existing: Query<(), With<ColliderBlobs>>,
    mut commands: Commands,
) {
    let ent = trigger.entity;
    let ColliderEvent::Rebuild(attr) = &trigger.value;

    if existing.contains(ent) {
        commands
            .entity(ent)
            .remove::<ColliderBlobs>()
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }

    match attr {
        ColliderAttr::Sphere(r) => {
            if let Some(c) = build_sphere(*r) {
                commands.entity(ent).insert(c);
            }
        }
        ColliderAttr::Capsule { height, radius } => {
            if let Some(c) = build_capsule(*height, *radius) {
                commands.entity(ent).insert(c);
            }
        }
        ColliderAttr::Cuboid { x, y, z } => {
            if let Some(c) = build_cuboid(*x, *y, *z) {
                commands.entity(ent).insert(c);
            }
        }
        ColliderAttr::Cylinder { height, radius } => {
            if let Some(c) = build_cylinder(*height, *radius) {
                commands.entity(ent).insert(c);
            }
        }
        ColliderAttr::ConvexHull(hash) => {
            let points_ent = commands
                .spawn((BlobDep(ent), BlobRequest(blake3::Hash::from_bytes(hash.0))))
                .id();
            commands
                .entity(ent)
                .insert(ColliderBlobs(ColliderBlobKind::ConvexHull {
                    points: points_ent,
                }));
        }
        ColliderAttr::Trimesh { vertices, indices } => {
            let vertex_ent = commands
                .spawn((
                    BlobDep(ent),
                    BlobRequest(blake3::Hash::from_bytes(vertices.0)),
                ))
                .id();
            let index_ent = commands
                .spawn((
                    BlobDep(ent),
                    BlobRequest(blake3::Hash::from_bytes(indices.0)),
                ))
                .id();
            commands
                .entity(ent)
                .insert(ColliderBlobs(ColliderBlobKind::Trimesh {
                    vertices: vertex_ent,
                    indices: index_ent,
                }));
        }
    }
}

pub fn on_collider_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    collider_blobs: Query<&ColliderBlobs>,
    mut blob_responses: Query<&mut BlobResponse>,
    mut commands: Commands,
) {
    let ent = trigger.entity;
    let Ok(blobs) = collider_blobs.get(ent) else {
        return;
    };

    let collider = match &blobs.0 {
        ColliderBlobKind::ConvexHull { points } => {
            let Ok(Some(bytes)) = blob_responses.get_mut(*points).map(|mut b| b.0.take()) else {
                warn!("convex hull blob not ready");
                return;
            };
            build_convex_hull(&bytes)
        }
        ColliderBlobKind::Trimesh { vertices, indices } => {
            let Ok(Some(vb)) = blob_responses.get_mut(*vertices).map(|mut b| b.0.take()) else {
                warn!("trimesh vertex blob not ready");
                return;
            };
            let Ok(Some(ib)) = blob_responses.get_mut(*indices).map(|mut b| b.0.take()) else {
                warn!("trimesh index blob not ready");
                return;
            };
            build_trimesh(&vb, &ib)
        }
    };

    if let Some(c) = collider {
        commands.entity(ent).insert(c);
    }

    commands
        .entity(ent)
        .remove::<ColliderBlobs>()
        .remove::<BlobDeps>()
        .remove::<BlobDepsLoaded>();
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
