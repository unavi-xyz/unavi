#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct Component;

impl Guest for Component {
    fn hello_world() -> String {
        bindings::unavi::ui::unavi_ui_types::new_bubble();
        "Hello, World!".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
