#[allow(warnings)]
mod bindings;

use bindings::{
    exports::wired::script::lifecycle::Guest as Script, wired::log::api::LogLevel, Guest,
};

use crate::bindings::wired::log::api::log;

struct Component;

impl Guest for Component {
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}

impl Script for Component {
    fn init() {
        bindings::unavi::ui::api::new_bubble();

        log(LogLevel::Info, "unavi-system Script initialized");

        println!("Component initialized");
    }

    fn update(delta_seconds: f32) {
        println!("Component updated with delta_seconds: {}", delta_seconds);
    }

    fn cleanup() {
        println!("Component cleaned up");
    }
}

bindings::export!(Component with_types_in bindings);
