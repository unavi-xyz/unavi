use bindings::{
    exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData},
    unavi::shapes::api::create_cuboid,
    wired::{
        gltf::{
            material::{create_material, Color},
            node::{create_node, Node},
        },
        math::types::Vec3,
    },
};
use glam::Quat;

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;
mod impls;

struct DataImpl {
    node: Node,
}

impl GuestData for DataImpl {}

struct Script;

impl Guest for Script {
    type Data = DataImpl;

    fn init() -> Data {
        log(LogLevel::Info, "Hello from example-wired-gltf!");

        let node = create_node();

        let mesh = create_cuboid(Vec3::splat(1.0));

        let material = create_material();
        material.set_color(Color {
            r: 1.0,
            g: 0.5,
            b: 0.9,
            a: 1.0,
        });

        // for primitive in mesh.list_primitives() {
        //     primitive.set_material(&material);
        // }

        node.set_mesh(&mesh);

        Data::new(DataImpl { node })
    }

    fn update(delta: f32, data: DataBorrow) {
        let data = data.get::<DataImpl>();

        let mut transform = data.node.transform();

        let mut quat = Quat::from_xyzw(
            transform.rotation.x,
            transform.rotation.y,
            transform.rotation.z,
            transform.rotation.w,
        );

        quat *= Quat::from_rotation_y(delta);

        transform.rotation.x = quat.x;
        transform.rotation.y = quat.y;
        transform.rotation.z = quat.z;
        transform.rotation.w = quat.w;

        data.node.set_transform(transform);
    }
}

bindings::export!(Script with_types_in bindings);
