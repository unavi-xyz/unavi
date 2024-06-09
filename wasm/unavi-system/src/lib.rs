use bindings::{
    exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData},
    wired::{
        gltf::node::{create_node, list_nodes},
        log::api::{log, LogLevel},
    },
};

#[allow(warnings)]
mod bindings;
mod impls;

struct DataImpl {}

impl GuestData for DataImpl {}

struct UnaviSystem;

impl Guest for UnaviSystem {
    type Data = DataImpl;

    fn init() -> Data {
        log(LogLevel::Info, "initializing...");

        let node = create_node();
        let found_nodes = list_nodes();

        if found_nodes.is_empty() {
            log(LogLevel::Error, "found 0 nodes")
        } else if found_nodes[0].id() != node.id() {
            log(LogLevel::Error, "created node not in found_nodes!")
        }

        log(LogLevel::Info, "initialized!");

        Data::new(DataImpl {})
    }

    fn update(data: DataBorrow) {
        let _data: &DataImpl = data.get();
    }
}

bindings::export!(UnaviSystem with_types_in bindings);
