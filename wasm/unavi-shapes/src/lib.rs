use bevy_mesh::{
    Indices as BevyIndices, Mesh as BevyMesh, PrimitiveTopology as BevyTopology,
    VertexAttributeValues,
};

use crate::{
    exports::unavi::shapes::api::Guest,
    wired::scene::types::{Indices, Mesh, PrimitiveTopology},
};

mod cuboid;

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type Cuboid = cuboid::CuboidWrapped;
}

fn convert_bevy_mesh(mut in_mesh: BevyMesh) -> Mesh {
    let out_mesh = Mesh::new();

    let topology = match in_mesh.primitive_topology() {
        BevyTopology::PointList => PrimitiveTopology::PointList,
        BevyTopology::LineList => PrimitiveTopology::LineList,
        BevyTopology::LineStrip => PrimitiveTopology::LineStrip,
        BevyTopology::TriangleList => PrimitiveTopology::TriangleList,
        BevyTopology::TriangleStrip => PrimitiveTopology::TriangleStrip,
    };
    out_mesh.set_topology(topology);

    if let Some(indices) = in_mesh.remove_indices() {
        let indices = match indices {
            BevyIndices::U16(value) => Indices::Half(value),
            BevyIndices::U32(value) => Indices::Full(value),
        };

        out_mesh.set_indices(Some(&indices));
    }

    for (attr, values) in in_mesh.attributes() {
        match *attr {
            BevyMesh::ATTRIBUTE_POSITION => {
                let VertexAttributeValues::Float32x3(values) = values else {
                    panic!("invalid values")
                };
                out_mesh.set_positions(Some(values.as_flattened()));
            }
            BevyMesh::ATTRIBUTE_NORMAL => {
                let VertexAttributeValues::Float32x3(values) = values else {
                    panic!("invalid values")
                };
                out_mesh.set_normals(Some(values.as_flattened()));
            }
            BevyMesh::ATTRIBUTE_TANGENT => {
                let VertexAttributeValues::Float32x4(values) = values else {
                    panic!("invalid values")
                };
                out_mesh.set_tangents(Some(values.as_flattened()));
            }
            BevyMesh::ATTRIBUTE_COLOR => {
                let VertexAttributeValues::Float32x4(values) = values else {
                    panic!("invalid values")
                };
                out_mesh.set_colors(Some(values.as_flattened()));
            }
            BevyMesh::ATTRIBUTE_UV_0 => {
                let VertexAttributeValues::Float32x2(values) = values else {
                    panic!("invalid values")
                };
                out_mesh.set_uv0(Some(values.as_flattened()));
            }
            BevyMesh::ATTRIBUTE_UV_1 => {
                let VertexAttributeValues::Float32x2(values) = values else {
                    panic!("invalid values")
                };
                out_mesh.set_uv1(Some(values.as_flattened()));
            }
            _ => {}
        }
    }

    out_mesh
}

export!(World);
