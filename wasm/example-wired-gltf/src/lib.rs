use bindings::{
    exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData},
    unavi::shapes::api::{create_cuboid, create_sphere},
    wired::{
        gltf::node::{create_node, Transform},
        math::types::Vec3,
    },
};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;
mod impls;

struct DataImpl {}

impl GuestData for DataImpl {}

struct Script;

impl Guest for Script {
    type Data = DataImpl;

    fn init() -> Data {
        log(LogLevel::Info, "Hello from example-wired-gltf!");

        let node = create_node();
        let mesh = create_cuboid(Vec3::splat(1.0));
        node.set_mesh(&mesh);

        let node = create_node();
        node.set_transform(Transform {
            translation: Vec3::splat(1.5),
            ..Default::default()
        });
        let mesh = create_sphere(0.5, 32, 18);
        node.set_mesh(&mesh);

        Data::new(DataImpl {})
    }

    fn update(_data: DataBorrow) {}
}

bindings::export!(Script with_types_in bindings);
