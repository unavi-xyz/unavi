use bindings::exports::wired::script::lifecycle::{Data, DataBorrow, Guest, GuestData};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;
mod nodes;

struct DataImpl {}

impl GuestData for DataImpl {}

struct Script;

impl Guest for Script {
    type Data = DataImpl;

    fn init() -> Data {
        log(LogLevel::Info, "Hello from script!");

        nodes::test_nodes();

        Data::new(DataImpl {})
    }

    fn update(_data: DataBorrow) {}
}

fn panic_log(err: &str) {
    log(LogLevel::Error, err);
    panic!("{}", err);
}

bindings::export!(Script with_types_in bindings);
