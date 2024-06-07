use ncollide3d::procedural::TriMesh;

use crate::bindings::wired::gltf::mesh::Primitive;

pub fn set_attributes(primitive: &Primitive, mesh: TriMesh<f32>) {
    let positions = mesh
        .coords
        .iter()
        .flat_map(|v| [v[0], v[1], v[2]])
        .collect::<Vec<_>>();
    primitive.set_positions(&positions);

    let normals = mesh
        .normals
        .as_ref()
        .unwrap()
        .iter()
        .flat_map(|v| [v[0], v[1], v[2]])
        .collect::<Vec<_>>();
    primitive.set_normals(&normals);

    let uvs = mesh
        .uvs
        .as_ref()
        .unwrap()
        .iter()
        .flat_map(|v| [v[0], v[1], v[2]])
        .collect::<Vec<_>>();
    primitive.set_uvs(&uvs);

    let indices = mesh.flat_indices();
    primitive.set_indices(&indices);
}
