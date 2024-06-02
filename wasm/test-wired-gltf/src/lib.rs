use bindings::{
    exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData},
    wired::gltf::node::{create_node, list_nodes, remove_node},
};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;

struct DataImpl {}

impl GuestData for DataImpl {}

struct Script;

impl Guest for Script {
    type Data = DataImpl;

    fn init() -> Data {
        log(LogLevel::Info, "Hello from script!");

        let node = create_node();
        let found_nodes = list_nodes();

        if found_nodes.len() != 1 {
            let err = format!("found list_nodes len: {}, expected 1", found_nodes.len());
            panic(&err);
        }

        if found_nodes[0].id() != node.id() {
            let err = format!(
                "found node id: {}, expected: {}",
                found_nodes[0].id(),
                node.id()
            );
            panic(&err);
        };

        remove_node(node);
        let found_nodes = list_nodes();

        if found_nodes.len() != 0 {
            let err = format!("found list_nodes len: {}, expected 0", found_nodes.len());
            panic(&err);
        }

        Data::new(DataImpl {})
    }

    fn update(_data: DataBorrow) {}
}

fn panic(err: &str) {
    log(LogLevel::Error, err);
    panic!("{}", err);
}

bindings::export!(Script with_types_in bindings);
