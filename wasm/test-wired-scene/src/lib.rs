use bindings::exports::wired::script::types::{Guest, GuestScript};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;
mod material;
mod mesh;
mod node;
mod property_tests;
mod scene;
mod wired_math_impls;
mod wired_scene_impls;

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        log(LogLevel::Info, "Called script construct!");

        material::test_material_api();
        mesh::test_mesh_api();
        node::test_node_api();
        scene::test_scene_api();

        Script
    }

    fn update(&self, _delta: f32) {
        log(LogLevel::Info, "Called script update!");
    }
}

fn panic_log(err: &str) {
    log(LogLevel::Error, err);
    panic!("{}", err);
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
