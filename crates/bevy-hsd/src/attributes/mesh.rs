use bevy::{
    asset::RenderAssetUsages,
    mesh::{
        Indices,
        MeshVertexAttribute,
        PrimitiveTopology,
        VertexAttributeValues,
    },
    platform::collections::HashMap,
    prelude::*,
};
use bytemuck::{
    Pod,
    PodCastError,
    try_cast_slice,
};
use hsd::{
    bounds::MAX_MESH_ELEMENTS,
    property::{
        Payload,
        name::PropName,
    },
    schema::mesh::{
        self,
        MeshIndices,
        MeshStream,
        Topology,
    },
};
use smol_str::SmolStr;
use thiserror::Error;

use crate::attributes::{
    AttributeParser,
    ParseError,
};

/// Assembled from `mesh/topology`, `mesh/indices` and `mesh/stream:<NAME>`,
/// each its own field so one stream update never decodes the others.
#[derive(Component, Debug, Clone, Default)]
pub struct MeshData {
    pub topology: Option<Topology>,
    pub indices:  Option<MeshIndices>,
    pub streams:  HashMap<SmolStr, MeshStream>,
}

pub struct MeshParser;

impl AttributeParser for MeshParser {
    fn group(&self) -> &'static str {
        mesh::GROUP
    }

    /// `topology` gates the whole mesh: without it there is nothing to build,
    /// so its removal tears down the mesh entirely.
    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        name: &PropName,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        if let Some(stream) = mesh::stream_of(name) {
            let value = payload.map(MeshStream::decode).transpose()?;
            let stream = SmolStr::new(stream);
            commands
                .entity(prim)
                .entry::<MeshData>()
                .or_default()
                .and_modify(move |mut data| match value {
                    Some(value) => {
                        data.streams.insert(stream, value);
                    }
                    None => {
                        data.streams.remove(&stream);
                    }
                });
            return Ok(());
        }

        match name.field() {
            Some("topology") => match payload.map(Topology::decode).transpose()? {
                Some(topology) => {
                    commands
                        .entity(prim)
                        .entry::<MeshData>()
                        .or_default()
                        .and_modify(move |mut data| data.topology = Some(topology));
                    commands.entity(prim).insert(Mesh3d::default());
                }
                None => {
                    commands.entity(prim).remove::<(MeshData, Mesh3d)>();
                }
            },
            Some("indices") => {
                let indices = payload.map(MeshIndices::decode).transpose()?;
                commands
                    .entity(prim)
                    .entry::<MeshData>()
                    .or_default()
                    .and_modify(move |mut data| data.indices = indices);
            }
            _ => {}
        }
        Ok(())
    }
}

/// Rebuilds whenever any field changes. Bevy's vertex-buffer assembly needs
/// every stream and the index buffer together, so there is no cheaper partial
/// rebuild once more than the topology has changed.
pub fn rebuild_mesh(
    changed: Query<(Entity, &MeshData), Changed<MeshData>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut commands: Commands,
) {
    for (prim, data) in &changed {
        match build_mesh(data) {
            Ok(mesh) => {
                let handle = mesh_assets.add(mesh);
                commands.entity(prim).insert(Mesh3d(handle));
            }
            Err(err) => {
                if !matches!(err, MeshRejected::NoPosition) {
                    warn!("rejected mesh: {err}");
                }
                commands.entity(prim).remove::<Mesh3d>();
            }
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

fn build_mesh(data: &MeshData) -> Result<Mesh, MeshRejected> {
    let mut mesh = Mesh::new(
        topology_to_primitive(data.topology.unwrap_or_default()),
        RenderAssetUsages::default(),
    );

    let positions = data
        .streams
        .get("POSITION")
        .ok_or(MeshRejected::NoPosition)?;
    let vertices = checked::<[f32; 3]>("POSITION", &positions.0)?.len();

    for (name, stream) in &data.streams {
        let Some((attr, kind)) = mesh_attr_id(name) else {
            continue;
        };
        let bytes = &stream.0;

        let values = match kind {
            VertexKind::Float32x2 => {
                VertexAttributeValues::Float32x2(checked::<[f32; 2]>(name, bytes)?)
            }
            VertexKind::Float32x3 => {
                VertexAttributeValues::Float32x3(checked::<[f32; 3]>(name, bytes)?)
            }
            VertexKind::Float32x4 => {
                VertexAttributeValues::Float32x4(checked::<[f32; 4]>(name, bytes)?)
            }
        };

        if values.len() != vertices {
            return Err(MeshRejected::LengthMismatch {
                name:     name.to_string(),
                len:      values.len(),
                expected: vertices,
            });
        }
        mesh.insert_attribute(attr, values);
    }

    if let Some(indices) = &data.indices {
        let indices = checked::<u32>("indices", &indices.0)?;
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

fn mesh_attr_id(name: &str) -> Option<(MeshVertexAttribute, VertexKind)> {
    match name {
        "COLOR" => Some((Mesh::ATTRIBUTE_COLOR, VertexKind::Float32x4)),
        "NORMAL" => Some((Mesh::ATTRIBUTE_NORMAL, VertexKind::Float32x3)),
        "POSITION" => Some((Mesh::ATTRIBUTE_POSITION, VertexKind::Float32x3)),
        "TANGENT" => Some((Mesh::ATTRIBUTE_TANGENT, VertexKind::Float32x4)),
        "UV_0" => Some((Mesh::ATTRIBUTE_UV_0, VertexKind::Float32x2)),
        "UV_1" => Some((Mesh::ATTRIBUTE_UV_1, VertexKind::Float32x2)),
        _ => {
            warn!("unknown mesh attribute: {name}");
            None
        }
    }
}

enum VertexKind {
    Float32x2,
    Float32x3,
    Float32x4,
}
