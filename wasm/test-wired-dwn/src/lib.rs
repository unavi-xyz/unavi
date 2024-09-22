use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    // wired::dwn::{
    //     api::{local_dwn, world_host_dwn},
    //     dwn::Query,
    // },
};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;

struct Script {
    // missing_id: Query,
    // found_id: Query,
}

impl GuestScript for Script {
    fn new() -> Self {
        log(LogLevel::Info, "Called script construct!");

        // let local = local_dwn();
        // let _host = world_host_dwn();
        //
        // let missing_id = local.query();
        // missing_id.set_record_id(Some("missing"));
        // let missing_id = missing_id.run();
        //
        // let found_id = local.query();
        // found_id.set_record_id(Some("test"));
        // let found_id = found_id.run();

        Script {
            // missing_id,
            // found_id,
        }
    }

    fn update(&self, _delta: f32) {
        log(LogLevel::Info, "Called script update!");

        // if !self.missing_id.finished() {
        //     if let Some(reply) = self.missing_id.poll() {
        //         if reply.status.code == 200 {
        //             panic_log("Invalid ID query returned 200");
        //         }
        //     }
        // }
        //
        // if !self.found_id.finished() {
        //     if let Some(reply) = self.found_id.poll() {
        //         if reply.status.code != 200 {
        //             panic_log("Failed to query record by ID");
        //         }
        //     }
        // }
    }
}

// fn panic_log(err: &str) {
//     log(LogLevel::Error, err);
//     panic!("{}", err);
// }

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
