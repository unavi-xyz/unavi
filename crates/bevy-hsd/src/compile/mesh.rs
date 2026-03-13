use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues},
    prelude::*,
};
use bevy_wds::{BlobDep, BlobDeps, BlobDepsLoaded, BlobRequest, BlobResponse};
use bytemuck::{Pod, PodCastError, try_cast_slice};
use bytes::Bytes;
use smol_str::SmolStr;

use crate::{CompiledMesh, compile::Uncompiled, data::HsdMesh};

pub fn parse_mesh_data(
    mut commands: Commands,
    meshes: Query<(Entity, &HsdMesh), Changed<HsdMesh>>,
) {
    for (ent, mesh) in &meshes {
        let mut attr_deps = Vec::new();

        for (name, hash) in &mesh.attributes {
            let dep = commands
                .spawn((
                    BlobDep { owner: ent },
                    BlobRequest(hash.0),
                    MeshAttrName(name.clone()),
                ))
                .id();
            attr_deps.push(dep);
        }

        let indices = mesh.indices.map(|hash| {
            commands
                .spawn((BlobDep { owner: ent }, BlobRequest(hash.0)))
                .id()
        });

        commands.entity(ent).insert(MeshParams {
            topology: mesh.topology.0,
            attr_deps,
            indices,
        });
    }
}

#[derive(Component)]
pub struct MeshAttrName(pub SmolStr);

#[derive(Component)]
#[require(BlobDeps)]
pub struct MeshParams {
    pub topology: PrimitiveTopology,
    pub attr_deps: Vec<Entity>,
    pub indices: Option<Entity>,
}

/// Map HSD attribute names to Bevy mesh attributes.
fn mesh_attr_id(name: &str) -> Option<(MeshVertexAttribute, MeshAttrKind)> {
    match name {
        "POSITION" => Some((Mesh::ATTRIBUTE_POSITION, MeshAttrKind::Float32x3)),
        "NORMAL" => Some((Mesh::ATTRIBUTE_NORMAL, MeshAttrKind::Float32x3)),
        "TANGENT" => Some((Mesh::ATTRIBUTE_TANGENT, MeshAttrKind::Float32x4)),
        "COLOR" => Some((Mesh::ATTRIBUTE_COLOR, MeshAttrKind::Float32x4)),
        "UV_0" => Some((Mesh::ATTRIBUTE_UV_0, MeshAttrKind::Float32x2)),
        "UV_1" => Some((Mesh::ATTRIBUTE_UV_1, MeshAttrKind::Float32x2)),
        _ => {
            warn!("unknown mesh attribute: {name}");
            None
        }
    }
}

enum MeshAttrKind {
    Float32x2,
    Float32x3,
    Float32x4,
}

pub fn compile_meshes(
    mut mesh_assets: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    loaded: Query<
        (Entity, &MeshParams, Option<&CompiledMesh>),
        (
            Or<(Added<BlobDepsLoaded>, Changed<MeshParams>, With<Uncompiled>)>,
            With<BlobDepsLoaded>,
        ),
    >,
    mut blobs: Query<&mut BlobResponse>,
    attr_names: Query<&MeshAttrName>,
) {
    for (ent, params, existing) in &loaded {
        commands.entity(ent).remove::<Uncompiled>();

        let mut mesh = Mesh::new(params.topology, RenderAssetUsages::all());

        // Insert indices.
        if let Some(idx_ent) = params.indices {
            let Ok(Some(bytes)) = blobs.get_mut(idx_ent).map(|mut b| b.0.take()) else {
                continue;
            };
            let indices = match bytes_to_vec::<u32>(&bytes) {
                Ok(s) => s,
                Err(err) => {
                    warn!(?err, "invalid indices");
                    continue;
                }
            };
            mesh.insert_indices(Indices::U32(indices));
        }

        // Insert attributes.
        for &dep_ent in &params.attr_deps {
            let Ok(name) = attr_names.get(dep_ent) else {
                warn!("attr name not found");
                continue;
            };
            let Ok(Some(bytes)) = blobs.get_mut(dep_ent).map(|mut b| b.0.take()) else {
                warn!("blob dep not found");
                continue;
            };

            let Some((attr, kind)) = mesh_attr_id(&name.0) else {
                continue;
            };

            match kind {
                MeshAttrKind::Float32x2 => match bytes_to_vec::<[f32; 2]>(&bytes) {
                    Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x2(v)),
                    Err(err) => {
                        warn!(?err, "invalid {} buffer", name.0);
                    }
                },
                MeshAttrKind::Float32x3 => match bytes_to_vec::<[f32; 3]>(&bytes) {
                    Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x3(v)),
                    Err(err) => {
                        warn!(?err, "invalid {} buffer", name.0);
                    }
                },
                MeshAttrKind::Float32x4 => match bytes_to_vec::<[f32; 4]>(&bytes) {
                    Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x4(v)),
                    Err(err) => {
                        warn!(?err, "invalid {} buffer", name.0);
                    }
                },
            }
        }

        // Update existing asset in-place to preserve handles in referencing nodes.
        if let Some(CompiledMesh(handle)) = existing {
            if let Some(asset) = mesh_assets.get_mut(handle) {
                *asset = mesh;
                commands
                    .entity(ent)
                    .remove::<BlobDeps>()
                    .remove::<BlobDepsLoaded>();
            } else {
                // Asset not ready?
                commands.entity(ent).insert(Uncompiled);
            }
        } else {
            let handle = asset_server.add(mesh);
            commands
                .entity(ent)
                .insert(CompiledMesh(handle))
                .remove::<BlobDeps>()
                .remove::<BlobDepsLoaded>();
        }
    }
}

fn bytes_to_vec<T: Pod>(bytes: &Bytes) -> Result<Vec<T>, PodCastError> {
    let slice = try_cast_slice::<u8, T>(bytes)?;
    Ok(slice.to_vec())
}
