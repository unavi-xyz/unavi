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
        let mut mesh = Mesh::new(
            topology_to_primitive(data.0.topology),
            RenderAssetUsages::default(),
        );

        if let Some(bytes) = slots.0.get(slots::MESH_INDICES) {
            match bytes_to_vec::<u32>(bytes) {
                Ok(v) => mesh.insert_indices(Indices::U32(v)),
                Err(err) => warn!(?err, "invalid indices buffer"),
            }
        }

        for (slot_name, bytes) in &slots.0 {
            let Some(name) = slots::mesh_attribute_name(slot_name) else {
                continue;
            };
            let Some((attr, kind)) = mesh_attr_id(name) else {
                continue;
            };

            match kind {
                MeshAttrKind::Float32x2 => match bytes_to_vec::<[f32; 2]>(bytes) {
                    Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x2(v)),
                    Err(err) => warn!(?err, "invalid {name} buffer"),
                },
                MeshAttrKind::Float32x3 => match bytes_to_vec::<[f32; 3]>(bytes) {
                    Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x3(v)),
                    Err(err) => warn!(?err, "invalid {name} buffer"),
                },
                MeshAttrKind::Float32x4 => match bytes_to_vec::<[f32; 4]>(bytes) {
                    Ok(v) => mesh.insert_attribute(attr, VertexAttributeValues::Float32x4(v)),
                    Err(err) => warn!(?err, "invalid {name} buffer"),
                },
            }
        }

        if !mesh.contains_attribute(Mesh::ATTRIBUTE_POSITION) {
            continue;
        }

        let handle = mesh_assets.add(mesh);
        commands.entity(prim).insert(Mesh3d(handle));
    }
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

fn bytes_to_vec<T: Pod>(bytes: &[u8]) -> Result<Vec<T>, PodCastError> {
    let slice = try_cast_slice::<u8, T>(bytes)?;
    Ok(slice.to_vec())
}
