use bindings::{
    exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData},
    wired::gltf::node::nodes,
    wired::log::api::{log, LogLevel},
};

#[allow(warnings)]
mod bindings;

struct DataImpl {}

impl GuestData for DataImpl {}

struct UnaviSystem;

impl Guest for UnaviSystem {
    type Data = DataImpl;

    fn init() -> Data {
        log(LogLevel::Info, "initialized");

        let _found_nodes = nodes();

        Data::new(DataImpl {})
    }

    fn update(data: DataBorrow) {
        let _data: &DataImpl = data.get();
    }
}

bindings::export!(UnaviSystem with_types_in bindings);
