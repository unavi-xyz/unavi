use attributes::set_attributes;
use bindings::{
    exports::unavi::shapes::api::Guest,
    wired::{math::types::Vec3, scene::mesh::Mesh},
};
use parry3d::shape::Ball;
use shapes::cuboid::make_cuboid;

mod attributes;
#[allow(warnings)]
mod bindings;
mod shapes;

struct Api;

impl Guest for Api {
    fn create_cuboid(size: Vec3) -> Mesh {
        let mesh = Mesh::new();
        let primitive = mesh.create_primitive();

        make_cuboid(&size, &primitive);

        mesh
    }

    fn create_sphere(radius: f32, sectors: u32, stacks: u32) -> Mesh {
        let mesh = Mesh::new();
        let primitive = mesh.create_primitive();

        let shape = Ball::new(radius);

        set_attributes(&primitive, shape.to_trimesh(sectors, stacks));

        mesh
    }
}

bindings::export!(Api with_types_in bindings);
