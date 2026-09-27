use std::mem::size_of;

use avian3d::prelude::Collider;
use bevy::prelude::*;
use bytemuck::pod_collect_to_vec;
use hsd::{
    bounds::MAX_MESH_STREAM_BYTES,
    property::{
        Payload,
        name::PropName,
    },
    schema::collider::{
        ColliderIndices,
        ColliderKind,
        ColliderVertices,
    },
};
use unavi_physics::{
    body::{
        DisabledCollider,
        insert_collider,
    },
    shape,
};

use crate::attributes::{
    ParseError,
    Pending,
    util::compute_global_transform,
};

/// Assembled from `collider/kind`, `collider/vertices` and `collider/indices`,
/// each its own field so a change to one never re-decodes the others.
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct ColliderData {
    pub kind:     Option<ColliderKind>,
    pub vertices: Option<ColliderVertices>,
    pub indices:  Option<ColliderIndices>,
}

#[derive(Component)]
pub struct HsdCollider;

/// `kind` gates the whole collider: without it nothing can be built, so its
/// removal tears down the collider entirely rather than leaving a half-built
/// one.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    match name.field() {
        Some("kind") => match payload.map(ColliderKind::decode).transpose()? {
            Some(kind) => {
                commands
                    .entity(prim)
                    .entry::<ColliderData>()
                    .or_default()
                    .and_modify(move |mut data| data.kind = Some(kind));
                commands
                    .entity(prim)
                    .insert((HsdCollider, Pending::<HsdCollider>::default()));
            }
            None => {
                commands.entity(prim).remove::<(
                    ColliderData,
                    HsdCollider,
                    Collider,
                    DisabledCollider,
                    Pending<HsdCollider>,
                )>();
            }
        },
        Some("vertices") => {
            let vertices = payload.map(ColliderVertices::decode).transpose()?;
            commands
                .entity(prim)
                .entry::<ColliderData>()
                .or_default()
                .and_modify(move |mut data| data.vertices = vertices);
            commands
                .entity(prim)
                .insert(Pending::<HsdCollider>::default());
        }
        Some("indices") => {
            let indices = payload.map(ColliderIndices::decode).transpose()?;
            commands
                .entity(prim)
                .entry::<ColliderData>()
                .or_default()
                .and_modify(move |mut data| data.indices = indices);
            commands
                .entity(prim)
                .insert(Pending::<HsdCollider>::default());
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn rebuild_collider(
    changed: Query<(Entity, &ColliderData), Changed<ColliderData>>,
    locals: Query<&Transform>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    for (prim, data) in &changed {
        commands
            .entity(prim)
            .remove::<(Collider, Pending<HsdCollider>)>();

        let Some(kind) = data.kind else {
            continue;
        };
        let seed = compute_global_transform(prim, &locals, &parents);

        let collider = match kind {
            ColliderKind::Sphere(r) => shape::sphere(r as f32),
            ColliderKind::Capsule { height, radius } => {
                shape::capsule(radius as f32, height as f32)
            }
            ColliderKind::Cuboid { x, y, z } => shape::cuboid(x as f32, y as f32, z as f32),
            ColliderKind::Cylinder { height, radius } => {
                shape::cylinder(radius as f32, height as f32)
            }
            ColliderKind::ConvexHull => {
                let Some(vertices) = data.vertices.as_ref() else {
                    continue;
                };
                build_convex_hull(&vertices.0)
            }
            ColliderKind::Trimesh => {
                let (Some(vertices), Some(indices)) = (&data.vertices, &data.indices) else {
                    continue;
                };
                build_trimesh(&vertices.0, &indices.0)
            }
        };

        if let Some(c) = collider {
            insert_collider(&mut commands, prim, c, &seed);
        }
    }
}

fn build_convex_hull(bytes: &[u8]) -> Option<Collider> {
    let points = cast_to_vec3("convex hull", bytes)?;
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
    let vertices = cast_to_vec3("trimesh vertices", vertex_bytes)?;
    let raw_indices = cast_to_indices("trimesh indices", index_bytes)?;
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

/// Collider buffers arrive over document sync; hull and trimesh construction
/// are superlinear in point count, so input is bounded and length-checked
/// before use.
fn cast_to_vec3(name: &str, bytes: &[u8]) -> Option<Vec<Vec3>> {
    let raw: Vec<[f32; 3]> = checked(name, bytes)?;
    Some(raw.into_iter().map(Vec3::from_array).collect())
}

fn cast_to_indices(name: &str, bytes: &[u8]) -> Option<Vec<[u32; 3]>> {
    checked(name, bytes)
}

fn checked<T: bytemuck::Pod>(name: &str, bytes: &[u8]) -> Option<Vec<T>> {
    if bytes.len() > MAX_MESH_STREAM_BYTES {
        warn!(
            "{name}: buffer is {} bytes, over the cap of {MAX_MESH_STREAM_BYTES}",
            bytes.len()
        );
        return None;
    }
    if !bytes.len().is_multiple_of(size_of::<T>()) {
        warn!("{name}: buffer is not a whole number of elements");
        return None;
    }
    Some(pod_collect_to_vec(bytes))
}
