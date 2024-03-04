#[allow(warnings)]
mod bindings;

use bindings;

struct Component;

impl Guest for Component {}

bindings::export!(Component with_types_in bindings);
