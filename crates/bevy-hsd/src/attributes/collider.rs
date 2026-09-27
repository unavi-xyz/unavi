use avian3d::prelude::Collider;
use bevy::prelude::*;
use bytemuck::{
    PodCastError,
    try_cast_slice,
};
use hsd::{
    bounds::MAX_MESH_ELEMENTS,
    property::{
        Payload,
        name::PropName,
    },
    schema::collider::{
        self,
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
    AttributeParser,
    ParseError,
    util::compute_global_transform,
};

/// Assembled from `collider/kind`, `collider/vertices` and `collider/indices`,
/// each its own field so a change to one never re-decodes the others.
#[derive(Component, Debug, Clone, Default)]
pub struct ColliderData {
    pub kind:     Option<ColliderKind>,
    pub vertices: Option<ColliderVertices>,
    pub indices:  Option<ColliderIndices>,
}

#[derive(Component)]
pub struct HsdCollider;

pub struct ColliderParser;

impl AttributeParser for ColliderParser {
    fn group(&self) -> &'static str {
        collider::GROUP
    }

    /// `kind` gates the whole collider: without it nothing can be built, so
    /// its removal tears down the collider entirely rather than leaving a
    /// half-built one.
    fn lifecycle(
        &self,
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
                    commands.entity(prim).insert(HsdCollider);
                }
                None => {
                    commands
                        .entity(prim)
                        .remove::<(ColliderData, HsdCollider, Collider, DisabledCollider)>();
                }
            },
            Some("vertices") => {
                let vertices = payload.map(ColliderVertices::decode).transpose()?;
                commands
                    .entity(prim)
                    .entry::<ColliderData>()
                    .or_default()
                    .and_modify(move |mut data| data.vertices = vertices);
            }
            Some("indices") => {
                let indices = payload.map(ColliderIndices::decode).transpose()?;
                commands
                    .entity(prim)
                    .entry::<ColliderData>()
                    .or_default()
                    .and_modify(move |mut data| data.indices = indices);
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn rebuild_collider(
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

/// Collider buffers arrive over document sync; hull and trimesh construction
/// are superlinear in point count, so input is bounded before use.
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
