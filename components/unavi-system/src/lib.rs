use bindings::exports::wired::script::lifecycle::Guest;

use crate::bindings::unavi::ui::api::new_bubble;

#[allow(warnings)]
mod bindings;

struct Component;

impl Guest for Component {
    fn init() {
        println!("Component initialized");

        let res = new_bubble();
        println!("Result: {}", res);
    }

    fn update(_delta_seconds: f32) {}

    fn cleanup() {
        println!("Component cleaned up");
    }
}

bindings::export!(Component with_types_in bindings);
