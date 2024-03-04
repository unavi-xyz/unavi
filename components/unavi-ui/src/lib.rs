#[allow(warnings)]
mod bindings;

use bindings::exports::unavi::ui::types::Guest;

struct Component;

impl Guest for Component {
    fn new_bubble() -> String {
        "bubble".into()
    }
}

bindings::export!(Component with_types_in bindings);
