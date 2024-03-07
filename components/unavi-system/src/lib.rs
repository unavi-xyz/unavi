use bindings::exports::wired::script::lifecycle::Guest;

#[allow(warnings)]
mod bindings;

struct Component;

impl Guest for Component {
    fn init() {
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
