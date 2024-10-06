use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    wired::dwn::{
        api::{user_dwn, world_host_dwn},
        records_query::RecordsQuery,
        records_write::RecordsWrite,
    },
};

use crate::bindings::wired::log::api::{log, LogLevel};

#[allow(warnings)]
mod bindings;

struct Script {
    missing_id: RecordsQuery,
    write: RecordsWrite,
    found_id: RecordsQuery,
}

impl GuestScript for Script {
    fn new() -> Self {
        log(LogLevel::Info, "Called script construct!");

        let user = user_dwn();
        let _ = world_host_dwn();

        let missing_id = {
            let builder = user.records_query();
            builder.set_record_id(Some("missing"));
            builder.run()
        };

        let write = {
            let builder = user.records_write();
            builder.run()
        };

        let found_id = {
            let builder = user.records_query();
            builder.set_record_id(Some("test"));
            builder.run()
        };

        Script {
            missing_id,
            write,
            found_id,
        }
    }

    fn update(&self, _delta: f32) {
        log(LogLevel::Info, "Called script update!");

        if !self.missing_id.finished() {
            if let Some(reply) = self.missing_id.poll() {
                if reply.status.code == 200 {
                    panic_log("Invalid ID query returned 200");
                }
            }
        }

        if !self.write.finished() {
            if let Some(reply) = self.write.poll() {
                if reply.status.code != 200 {
                    panic_log(&format!("Write query returned {}", reply.status.code));
                }
            }
        }

        if !self.found_id.finished() {
            if let Some(reply) = self.found_id.poll() {
                if reply.status.code != 200 {
                    panic_log(&format!("Found ID query returned {}", reply.status.code));
                }
            }
        }
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
