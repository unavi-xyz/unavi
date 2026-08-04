use bevy::{
    asset::RenderAssetUsages,
    mesh::{
        Indices,
        MeshVertexAttribute,
        PrimitiveTopology,
        VertexAttributeValues,
    },
    prelude::*,
};
use bevy_wds::blob::{
    deps::{
        BlobDep,
        BlobDeps,
        BlobDepsLoaded,
    },
    request::{
        BlobRequest,
        BlobResponse,
    },
};
use bytemuck::{
    Pod,
    PodCastError,
    try_cast_slice,
};
use bytes::Bytes;
use hsd::attributes::{
    Attribute,
    mesh::{
        MeshAttr,
        Topology,
    },
    slots,
};
use smol_str::SmolStr;

use crate::{
    HsdBulk,
    attributes::{
        AttributeParser,
        ParseError,
    },
};

#[derive(Component, Debug, Clone, Copy)]
pub struct MeshData(pub MeshAttr);

#[derive(Component)]
pub struct MeshAttrName(pub SmolStr);

#[derive(Component)]
#[require(BlobDeps)]
pub struct MeshBlobs {
    pub topology: PrimitiveTopology,
    pub attrs:    Vec<Entity>,
    pub indices:  Option<Entity>,
}

#[derive(Component)]
#[relationship(relationship_target = MeshBlobsChild)]
pub struct MeshBlobsOwner(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = MeshBlobsOwner, linked_spawn)]
pub struct MeshBlobsChild(Entity);

pub struct MeshParser;

impl AttributeParser for MeshParser {
    fn key(&self) -> &'static str {
        MeshAttr::KEY
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
                    .insert((MeshData(MeshAttr::decode(payload)?), Mesh3d::default()));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<(MeshData, MeshBlobsChild, Mesh3d)>();
            }
        }
        Ok(())
    }
}

/// Rebuilds on either half changing, since the topology attribute and the
/// vertex buffers are separate entries and arrive in no particular order.
pub fn rebuild_mesh(
    changed: Query<(Entity, &MeshData, &HsdBulk), Or<(Changed<MeshData>, Changed<HsdBulk>)>>,
    mut commands: Commands,
) {
    for (prim, data, bulk) in &changed {
        commands.entity(prim).remove::<MeshBlobsChild>();

        let child = commands.spawn(MeshBlobsOwner(prim)).id();

        let attrs = bulk
            .0
            .iter()
            .filter_map(|(slot, hash)| {
                let name = slots::mesh_attribute_name(slot)?;
                Some(
                    commands
                        .spawn((
                            BlobDep(child),
                            BlobRequest(blake3::Hash::from_bytes(hash.0)),
                            MeshAttrName(SmolStr::new(name)),
                        ))
                        .id(),
                )
            })
            .collect();

        let indices = bulk.0.get(slots::MESH_INDICES).map(|hash| {
            commands
                .spawn((
                    BlobDep(child),
                    BlobRequest(blake3::Hash::from_bytes(hash.0)),
                ))
                .id()
        });

        commands.entity(child).insert(MeshBlobs {
            topology: topology_to_primitive(data.0.topology),
            attrs,
            indices,
        });
    }
}

pub fn on_mesh_blobs_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    mesh_blobs: Query<(&MeshBlobs, &MeshBlobsOwner)>,
    mut blob_responses: Query<&mut BlobResponse>,
    attr_names: Query<&MeshAttrName>,
    mut mesh3d: Query<&mut Mesh3d>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    let child = trigger.entity;
    let Ok((params, owner)) = mesh_blobs.get(child) else {
        return;
    };
    let prim = owner.0;

    let mut mesh = Mesh::new(params.topology, RenderAssetUsages::default());

    if let Some(idx_ent) = params.indices {
        let Ok(Some(bytes)) = blob_responses.get_mut(idx_ent).map(|mut b| b.0.take()) else {
            warn!("indices blob not found");
            commands.entity(child).try_despawn();
            return;
        };
        match bytes_to_vec::<u32>(&bytes) {
            Ok(v) => mesh.insert_indices(Indices::U32(v)),
            Err(err) => {
                warn!(?err, "invalid indices buffer");
                commands.entity(child).try_despawn();
                return;
            }
        }
    }

    for &dep_ent in &params.attrs {
        let Ok(name) = attr_names.get(dep_ent) else {
            warn!("attr name not found");
            continue;
        };
        let Ok(Some(bytes)) = blob_responses.get_mut(dep_ent).map(|mut b| b.0.take()) else {
            warn!("blob dep not loaded");
            continue;
        };

        let Some((attr, kind)) = mesh_attr_id(&name.0) else {
            continue;
        };

        match kind {
            MeshAttrKind::Float32x2 => match bytes_to_vec::<[f32; 2]>(&bytes) {
                Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x2(v)),
                Err(err) => warn!(?err, "invalid {} buffer", name.0),
            },
            MeshAttrKind::Float32x3 => match bytes_to_vec::<[f32; 3]>(&bytes) {
                Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x3(v)),
                Err(err) => warn!(?err, "invalid {} buffer", name.0),
            },
            MeshAttrKind::Float32x4 => match bytes_to_vec::<[f32; 4]>(&bytes) {
                Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x4(v)),
                Err(err) => warn!(?err, "invalid {} buffer", name.0),
            },
        }
    }

    if !mesh.contains_attribute(Mesh::ATTRIBUTE_POSITION) {
        commands.entity(child).try_despawn();
        return;
    }

    let handle = mesh_assets.add(mesh);
    if let Ok(mut mesh3d) = mesh3d.get_mut(prim) {
        mesh3d.0 = handle;
    } else {
        commands.entity(prim).insert(Mesh3d(handle));
    }

    commands.entity(child).try_despawn();
}

const fn topology_to_primitive(t: Topology) -> PrimitiveTopology {
    match t {
        Topology::PointList => PrimitiveTopology::PointList,
        Topology::LineList => PrimitiveTopology::LineList,
        Topology::LineStrip => PrimitiveTopology::LineStrip,
        Topology::TriangleList => PrimitiveTopology::TriangleList,
        Topology::TriangleStrip => PrimitiveTopology::TriangleStrip,
    }
}

fn mesh_attr_id(name: &str) -> Option<(MeshVertexAttribute, MeshAttrKind)> {
    match name {
        "COLOR" => Some((Mesh::ATTRIBUTE_COLOR, MeshAttrKind::Float32x4)),
        "NORMAL" => Some((Mesh::ATTRIBUTE_NORMAL, MeshAttrKind::Float32x3)),
        "POSITION" => Some((Mesh::ATTRIBUTE_POSITION, MeshAttrKind::Float32x3)),
        "TANGENT" => Some((Mesh::ATTRIBUTE_TANGENT, MeshAttrKind::Float32x4)),
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

fn bytes_to_vec<T: Pod>(bytes: &Bytes) -> Result<Vec<T>, PodCastError> {
    let slice = try_cast_slice::<u8, T>(bytes)?;
    Ok(slice.to_vec())
}
