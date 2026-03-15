use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};
use bytemuck::{PodCastError, try_cast_slice};
use bytes::Bytes;

use crate::data::HsdCollider;

pub fn parse_collider_data(
    mut commands: Commands,
    colliders: Query<(Entity, &HsdCollider), Added<HsdCollider>>,
) {
    for (ent, data) in &colliders {
        #[expect(clippy::cast_possible_truncation)]
        match data {
            HsdCollider::Capsule {
                radius,
                half_height,
            } => {
                commands
                    .entity(ent)
                    .insert(Collider::capsule(*radius as f32, *half_height as f32));
            }
            HsdCollider::Cuboid { x, y, z } => {
                commands
                    .entity(ent)
                    .insert(Collider::cuboid(*x as f32, *y as f32, *z as f32));
            }
            HsdCollider::Cylinder {
                radius,
                half_height,
            } => {
                commands
                    .entity(ent)
                    .insert(Collider::cylinder(*radius as f32, *half_height as f32));
            }
            HsdCollider::Sphere(r) => {
                commands.entity(ent).insert(Collider::sphere(*r as f32));
            }
            HsdCollider::ConvexHull(hash) => {
                let blob_ent = commands
                    .spawn((BlobDep { owner: ent }, BlobRequest(hash.0)))
                    .id();
                commands
                    .entity(ent)
                    .insert(ColliderParams::ConvexHull { blob_ent });
            }
            HsdCollider::Trimesh { vertices, indices } => {
                let verts_ent = commands
                    .spawn((BlobDep { owner: ent }, BlobRequest(vertices.0)))
                    .id();
                let idx_ent = commands
                    .spawn((BlobDep { owner: ent }, BlobRequest(indices.0)))
                    .id();
                commands
                    .entity(ent)
                    .insert(ColliderParams::Trimesh { verts_ent, idx_ent });
            }
        }
    }
}

#[derive(Component)]
#[require(BlobDeps)]
pub enum ColliderParams {
    ConvexHull { blob_ent: Entity },
    Trimesh { verts_ent: Entity, idx_ent: Entity },
}

pub fn compile_colliders(
    mut commands: Commands,
    loaded: Query<(Entity, &ColliderParams), (Added<BlobDepsLoaded>, With<ColliderParams>)>,
    mut blobs: Query<&mut BlobResponse>,
) {
    for (ent, params) in &loaded {
        let collider = match params {
            ColliderParams::ConvexHull { blob_ent } => {
                let Ok(Some(bytes)) = blobs.get_mut(*blob_ent).map(|mut b| b.0.take()) else {
                    continue;
                };
                match cast_vec3(&bytes) {
                    Ok(points) => Collider::convex_hull(points).unwrap_or_default(),
                    Err(err) => {
                        warn!(?err, "invalid convex hull points");
                        continue;
                    }
                }
            }
            ColliderParams::Trimesh { verts_ent, idx_ent } => {
                let Ok(Some(vbytes)) = blobs.get_mut(*verts_ent).map(|mut b| b.0.take()) else {
                    continue;
                };
                let Ok(Some(ibytes)) = blobs.get_mut(*idx_ent).map(|mut b| b.0.take()) else {
                    continue;
                };
                let verts = match cast_vec3(&vbytes) {
                    Ok(v) => v,
                    Err(err) => {
                        warn!(?err, "invalid trimesh vertices");
                        continue;
                    }
                };
                let indices = match try_cast_slice::<u8, [u32; 3]>(&ibytes) {
                    Ok(s) => s.to_vec(),
                    Err(err) => {
                        warn!(?err, "invalid trimesh indices");
                        continue;
                    }
                };
                Collider::trimesh(verts, indices)
            }
        };

        debug!("compiled collider {ent}");
        commands
            .entity(ent)
            .insert(collider)
            .remove::<ColliderParams>()
            .remove::<BlobDeps>()
            .remove::<BlobDepsLoaded>();
    }
}

fn cast_vec3(bytes: &Bytes) -> Result<Vec<Vec3>, PodCastError> {
    let pts = try_cast_slice::<u8, [f32; 3]>(bytes)?;
    Ok(pts.iter().map(|&[x, y, z]| Vec3::new(x, y, z)).collect())
}
