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
use hsd::{
    attributes::mesh::{
        self,
        MeshIndices,
        MeshStream,
        Topology,
    },
    property::{
        Payload,
        name::PropName,
    },
};
use smol_str::SmolStr;
use thiserror::Error;

use crate::attributes::{
    buffer::{
        BufferError,
        cast_buffer,
    },
    update_data,
};

/// Assembled from the `mesh/topology`, `mesh/indices` and
/// `mesh/stream:<NAME>` fields.
#[derive(Component, Debug, Clone, Default)]
pub struct MeshData {
    pub topology: Option<Topology>,
    pub indices:  Option<MeshIndices>,
    pub streams:  HashMap<SmolStr, MeshStream>,
}

/// Removing `topology` tears down the whole mesh.
pub fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
    if let Some(stream) = mesh::stream_of(name) {
        let value = payload.map(MeshStream::decode).transpose()?;
        let stream = SmolStr::new(stream);
        update_data::<MeshData>(commands, prim, move |data| match value {
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
                update_data::<MeshData>(commands, prim, move |data| data.topology = Some(topology));
            }
            None => {
                commands.entity(prim).remove::<(MeshData, Mesh3d)>();
            }
        },
        Some("indices") => {
            let indices = payload.map(MeshIndices::decode).transpose()?;
            update_data::<MeshData>(commands, prim, move |data| data.indices = indices);
        }
        _ => {}
    }
    Ok(())
}

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

/// Why peer-written mesh buffers were refused before reaching the GPU.
#[derive(Debug, Error)]
enum MeshRejected {
    #[error("no POSITION attribute")]
    NoPosition,
    #[error("{name}: {source}")]
    Buffer { name: String, source: BufferError },
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
    let vertices = cast_buffer::<[f32; 3]>(&positions.0)
        .map_err(|source| MeshRejected::Buffer {
            name: "POSITION".to_owned(),
            source,
        })?
        .len();

    for (name, stream) in &data.streams {
        let Some((attr, kind)) = mesh_attr_id(name) else {
            continue;
        };
        let bytes = &stream.0;

        let values = match kind {
            VertexKind::Float32x2 => {
                VertexAttributeValues::Float32x2(cast_buffer(bytes).map_err(|source| {
                    MeshRejected::Buffer {
                        name: name.to_string(),
                        source,
                    }
                })?)
            }
            VertexKind::Float32x3 => {
                VertexAttributeValues::Float32x3(cast_buffer(bytes).map_err(|source| {
                    MeshRejected::Buffer {
                        name: name.to_string(),
                        source,
                    }
                })?)
            }
            VertexKind::Float32x4 => {
                VertexAttributeValues::Float32x4(cast_buffer(bytes).map_err(|source| {
                    MeshRejected::Buffer {
                        name: name.to_string(),
                        source,
                    }
                })?)
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
        let indices = cast_buffer::<u32>(&indices.0).map_err(|source| MeshRejected::Buffer {
            name: "indices".to_owned(),
            source,
        })?;
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
