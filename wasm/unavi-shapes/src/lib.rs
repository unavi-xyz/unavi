use attributes::set_attributes;
use bindings::{
    exports::unavi::shapes::api::Guest,
    wired::{
        gltf::mesh::{create_mesh, Mesh},
        math::types::Vec3,
    },
};
use ncollide3d::na::Vector3;

mod attributes;
#[allow(warnings)]
mod bindings;

struct UnaviShapes;

impl Guest for UnaviShapes {
    fn create_cuboid(size: Vec3) -> Mesh {
        let mesh = create_mesh();
        let primitive = mesh.create_primitive();

        let extents = Vector3::new(size.x, size.y, size.z);
        let shape = ncollide3d::procedural::cuboid(&extents);

        set_attributes(&primitive, shape);

        mesh
    }

    fn create_sphere(radius: f32, sectors: u32, stacks: u32) -> Mesh {
        let mesh = create_mesh();
        let primitive = mesh.create_primitive();

        let shape = ncollide3d::procedural::sphere(radius * 2.0, sectors, stacks, true);

        set_attributes(&primitive, shape);

        mesh
    }
}

bindings::export!(UnaviShapes with_types_in bindings);
