use bindings::{
    exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData},
    wired::gltf::node::{create_node, nodes},
};

#[allow(warnings)]
mod bindings;

struct DataImpl {}

impl GuestData for DataImpl {}

struct Script;

impl Guest for Script {
    type Data = DataImpl;

    fn init() -> Data {
        let node = create_node();
        let found_nodes = nodes();

        assert_eq!(found_nodes.len(), 1);
        assert_eq!(found_nodes[0].id(), node.id());

        Data::new(DataImpl {})
    }

    fn update(_data: DataBorrow) {}
}

bindings::export!(Script with_types_in bindings);
