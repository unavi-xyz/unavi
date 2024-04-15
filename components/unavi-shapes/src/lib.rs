#[allow(warnings)]
mod bindings;

use bindings::exports::unavi::shapes::shapes::Guest;

struct Component;

impl Guest for Component {
    fn new_cube() -> String {
        println!("Creating a new cube");
        "cube".into()
    }
}

bindings::export!(Component with_types_in bindings);
