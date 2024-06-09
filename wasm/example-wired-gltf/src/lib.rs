use bindings::{
    exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData},
    unavi::shapes::api::create_cuboid,
    wired::{gltf::node::create_node, math::types::Vec3},
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
        node.set_mesh(Some(&mesh));

        Data::new(DataImpl {})
    }

    fn update(_data: DataBorrow) {}
}

bindings::export!(Script with_types_in bindings);
