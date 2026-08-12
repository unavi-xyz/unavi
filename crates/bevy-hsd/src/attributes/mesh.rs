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
use bytemuck::{
    Pod,
    PodCastError,
    try_cast_slice,
};
use hsd::attributes::{
    Attribute,
    mesh::{
        MeshAttr,
        Topology,
    },
    slots,
};
use thiserror::Error;
use unavi_quota::limits::MAX_MESH_ELEMENTS;

use crate::{
    HsdSlots,
    attributes::{
        AttributeParser,
        ParseError,
    },
};

#[derive(Component, Debug, Clone, Copy)]
pub struct MeshData(pub MeshAttr);

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
                commands.entity(prim).remove::<(MeshData, Mesh3d)>();
            }
        }
        Ok(())
    }
}

/// Rebuilds on either half changing, since the topology attribute and the
/// vertex buffers are separate entries and arrive in no particular order.
pub fn rebuild_mesh(
    changed: Query<(Entity, &MeshData, &HsdSlots), Or<(Changed<MeshData>, Changed<HsdSlots>)>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (prim, data, slots) in &changed {
        match build_mesh(data.0.topology, slots) {
            Ok(mesh) => {
                let handle = mesh_assets.add(mesh);
                commands.entity(prim).insert(Mesh3d(handle));
            }
            Err(MeshRejected::NoPosition) => {}
            Err(err) => warn!("rejected mesh: {err}"),
        }
    }
}

/// Why a document's mesh buffers were refused.
///
/// Buffers arrive over document sync from a peer, so none of the GPU's
/// invariants can be assumed: an index past the vertex count is an
/// out-of-bounds read at draw time, and attributes of differing lengths fail
/// Bevy's vertex-buffer assembly.
#[derive(Debug, Error)]
enum MeshRejected {
    #[error("no POSITION attribute")]
    NoPosition,
    #[error("buffer is not a whole number of elements: {0}")]
    Cast(PodCastError),
    #[error("{name} buffer is {len} bytes, over the cap of {MAX_MESH_ELEMENTS}")]
    TooLarge { name: String, len: usize },
    #[error("{name} has {len} vertices, but POSITION has {expected}")]
    LengthMismatch {
        name:     String,
        len:      usize,
        expected: usize,
    },
    #[error("index {index} is past the {vertices} vertices in the mesh")]
    IndexOutOfBounds { index: u32, vertices: usize },
}

fn build_mesh(topology: Topology, slots: &HsdSlots) -> Result<Mesh, MeshRejected> {
    let mut mesh = Mesh::new(
        topology_to_primitive(topology),
        RenderAssetUsages::default(),
    );

    let positions = slots
        .0
        .get(slots::mesh_attribute("POSITION").as_str())
        .ok_or(MeshRejected::NoPosition)?;
    let vertices = checked::<[f32; 3]>("POSITION", positions)?.len();

    for (slot_name, bytes) in &slots.0 {
        let Some(name) = slots::mesh_attribute_name(slot_name) else {
            continue;
        };
        let Some((attr, kind)) = mesh_attr_id(name) else {
            continue;
        };

        let values = match kind {
            MeshAttrKind::Float32x2 => {
                VertexAttributeValues::Float32x2(checked::<[f32; 2]>(name, bytes)?)
            }
            MeshAttrKind::Float32x3 => {
                VertexAttributeValues::Float32x3(checked::<[f32; 3]>(name, bytes)?)
            }
            MeshAttrKind::Float32x4 => {
                VertexAttributeValues::Float32x4(checked::<[f32; 4]>(name, bytes)?)
            }
        };

        if values.len() != vertices {
            return Err(MeshRejected::LengthMismatch {
                name:     name.to_owned(),
                len:      values.len(),
                expected: vertices,
            });
        }
        mesh.insert_attribute(attr, values);
    }

    if let Some(bytes) = slots.0.get(slots::MESH_INDICES) {
        let indices = checked::<u32>("indices", bytes)?;
        if let Some(&index) = indices
            .iter()
            .find(|&&i| usize::try_from(i).unwrap_or(usize::MAX) >= vertices)
        {
            return Err(MeshRejected::IndexOutOfBounds { index, vertices });
        }
        mesh.insert_indices(Indices::U32(indices));
    }

    Ok(mesh)
}

fn checked<T: Pod>(name: &str, bytes: &[u8]) -> Result<Vec<T>, MeshRejected> {
    if bytes.len() > MAX_MESH_ELEMENTS {
        return Err(MeshRejected::TooLarge {
            name: name.to_owned(),
            len:  bytes.len(),
        });
    }
    let slice = try_cast_slice::<u8, T>(bytes).map_err(MeshRejected::Cast)?;
    Ok(slice.to_vec())
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
