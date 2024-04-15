#[allow(warnings)]
mod bindings;

use bindings::{
    exports::unavi::shapes::shapes::Guest,
    wired::{
        gltf::mesh::{create_mesh, Mesh},
        math::types::Vec3,
    },
};

mod cuboid;

struct Component;

impl Guest for Component {
    fn create_cuboid(half_size: Vec3) -> Mesh {
        let mesh = create_mesh();
        mesh.set_name("Cube");

        let vertices = cuboid::vertices(half_size);

        let primitive = mesh.create_primitive();
        primitive.set_indices(&vertices.indices);
        primitive.set_positions(&vertices.positions);
        primitive.set_normals(&vertices.normals);
        primitive.set_uvs(&vertices.uvs);

        mesh
    }
}

bindings::export!(Component with_types_in bindings);
