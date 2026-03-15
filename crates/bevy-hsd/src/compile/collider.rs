use avian3d::prelude::Collider;
use bevy::prelude::*;
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};
use bytemuck::try_cast_slice;
use bytes::Bytes;

use crate::data::HsdCollider;

pub fn parse_collider_data(
    mut commands: Commands,
    colliders: Query<(Entity, &HsdCollider), Changed<HsdCollider>>,
) {
    for (ent, data) in &colliders {
        #[expect(clippy::cast_possible_truncation)]
        match data {
            HsdCollider::Capsule {
                radius,
                half_height,
            } => {
                let (r, h) = (*radius as f32, *half_height as f32);
                if !valid_positive(r) || !valid_nonneg(h) {
                    warn!("invalid capsule dims ({r}, {h}), skipping collider {ent}");
                    continue;
                }
                commands.entity(ent).insert(Collider::capsule(r, h));
            }
            HsdCollider::Cuboid { x, y, z } => {
                let (x, y, z) = (*x as f32, *y as f32, *z as f32);
                if !valid_positive(x) || !valid_positive(y) || !valid_positive(z) {
                    warn!("invalid cuboid dims ({x}, {y}, {z}), skipping collider {ent}");
                    continue;
                }
                commands.entity(ent).insert(Collider::cuboid(x, y, z));
            }
            HsdCollider::Cylinder {
                radius,
                half_height,
            } => {
                let (r, h) = (*radius as f32, *half_height as f32);
                if !valid_positive(r) || !valid_nonneg(h) {
                    warn!("invalid cylinder dims ({r}, {h}), skipping collider {ent}");
                    continue;
                }
                commands.entity(ent).insert(Collider::cylinder(r, h));
            }
            HsdCollider::Sphere(radius) => {
                let r = *radius as f32;
                if !valid_positive(r) {
                    warn!("invalid sphere radius ({r}), skipping collider {ent}");
                    continue;
                }
                commands.entity(ent).insert(Collider::sphere(r));
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
                let points = match cast_vec3_validated(&bytes) {
                    Ok(pts) => pts,
                    Err(e) => {
                        warn!("invalid convex hull data for {ent}: {e}");
                        continue;
                    }
                };
                let Some(c) = Collider::convex_hull(points) else {
                    warn!("failed to build convex hull for {ent}");
                    continue;
                };
                c
            }
            ColliderParams::Trimesh { verts_ent, idx_ent } => {
                let Ok(Some(vbytes)) = blobs.get_mut(*verts_ent).map(|mut b| b.0.take()) else {
                    continue;
                };
                let Ok(Some(ibytes)) = blobs.get_mut(*idx_ent).map(|mut b| b.0.take()) else {
                    continue;
                };
                let verts = match cast_vec3_validated(&vbytes) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("invalid trimesh vertices for {ent}: {e}");
                        continue;
                    }
                };
                let indices = match try_cast_slice::<u8, [u32; 3]>(&ibytes) {
                    Ok(s) => s.to_vec(),
                    Err(err) => {
                        warn!(?err, "invalid trimesh indices for {ent}");
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

fn valid_positive(v: f32) -> bool {
    v.is_finite() && v > 0.0
}

fn valid_nonneg(v: f32) -> bool {
    v.is_finite() && v >= 0.0
}

fn cast_vec3_validated(bytes: &Bytes) -> Result<Vec<Vec3>, String> {
    let pts = try_cast_slice::<u8, [f32; 3]>(bytes).map_err(|e| format!("cast error: {e:?}"))?;
    for &[x, y, z] in pts {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(format!("non-finite value in point ({x}, {y}, {z})"));
        }
    }
    Ok(pts.iter().map(|&[x, y, z]| Vec3::new(x, y, z)).collect())
}
