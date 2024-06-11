use bindings::exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;
mod impls;
mod material;
mod mesh;
mod node;
mod property_tests;

struct DataImpl {}

impl GuestData for DataImpl {}

struct Script;

impl Guest for Script {
    type Data = DataImpl;

    fn init() -> Data {
        log(LogLevel::Info, "Hello from script!");

        material::test_material_api();
        mesh::test_mesh_api();
        node::test_node_api();

        Data::new(DataImpl {})
    }

    fn update(_delta: f32, _data: DataBorrow) {}
}

fn panic_log(err: &str) {
    log(LogLevel::Error, err);
    panic!("{}", err);
}

bindings::export!(Script with_types_in bindings);
