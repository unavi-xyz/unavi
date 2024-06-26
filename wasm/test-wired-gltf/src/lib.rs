use bindings::exports::wired::script::types::{Guest, GuestScript};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;
mod material;
mod mesh;
mod node;
mod property_tests;
mod wired_gltf_impls;
mod wired_math_impls;

#[derive(Default)]
struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        log(LogLevel::Info, "Called script construct!");

        material::test_material_api();
        mesh::test_mesh_api();
        node::test_node_api();

        Script::default()
    }

    fn update(&self, _delta: f32) {
        log(LogLevel::Info, "Called script update!");
    }
}

fn panic_log(err: &str) {
    log(LogLevel::Error, err);
    panic!("{}", err);
}

struct Api;

impl Guest for Api {
    type Script = Script;
}

bindings::export!(Api with_types_in bindings);
