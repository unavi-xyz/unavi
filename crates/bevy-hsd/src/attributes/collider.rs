use avian3d::prelude::Collider;
use bevy::prelude::*;
use hsd::{
    attributes::collider::{
        ColliderIndices,
        ColliderKind,
        ColliderVertices,
    },
    property::{
        Payload,
        name::PropName,
    },
};
use unavi_physics::{
    body::{
        DisabledCollider,
        insert_collider,
    },
    shape,
};

use crate::{
    attributes::{
        buffer::cast_buffer,
        update_data,
    },
    hierarchy::global_transform,
};

/// Assembled from the `collider/kind`, `collider/vertices` and
/// `collider/indices` fields.
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct ColliderData {
    pub kind:     Option<ColliderKind>,
    pub vertices: Option<ColliderVertices>,
    pub indices:  Option<ColliderIndices>,
}

/// Removing `kind` tears down the whole collider.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    match name.field() {
        Some("kind") => match payload.map(ColliderKind::decode).transpose()? {
            Some(kind) => {
                update_data::<ColliderData>(commands, prim, move |data| data.kind = Some(kind));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<(ColliderData, Collider, DisabledCollider)>();
            }
        },
        Some("vertices") => {
            let vertices = payload.map(ColliderVertices::decode).transpose()?;
            update_data::<ColliderData>(commands, prim, move |data| data.vertices = vertices);
        }
        Some("indices") => {
            let indices = payload.map(ColliderIndices::decode).transpose()?;
            update_data::<ColliderData>(commands, prim, move |data| data.indices = indices);
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
        commands.entity(prim).remove::<Collider>();

        let Some(kind) = data.kind else {
            continue;
        };
        let seed = global_transform(prim, &locals, &parents);

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

/// Bounded, since hull and trimesh construction are superlinear in point
/// count.
fn cast_to_vec3(name: &str, bytes: &[u8]) -> Option<Vec<Vec3>> {
    let raw: Vec<[f32; 3]> = cast_buffer(bytes)
        .inspect_err(|err| warn!(%name, %err, "collider buffer rejected"))
        .ok()?;
    Some(raw.into_iter().map(Vec3::from_array).collect())
}

fn cast_to_indices(name: &str, bytes: &[u8]) -> Option<Vec<[u32; 3]>> {
    cast_buffer(bytes)
        .inspect_err(|err| warn!(%name, %err, "collider buffer rejected"))
        .ok()
}
