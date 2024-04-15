#[allow(warnings)]
mod bindings;

use bindings::exports::unavi::mirror::api::Guest;

struct Component;

impl Guest for Component {
    fn new_mirror() -> String {
        println!("Creating a new mirror");
        "mirror".into()
    }
}

bindings::export!(Component with_types_in bindings);
