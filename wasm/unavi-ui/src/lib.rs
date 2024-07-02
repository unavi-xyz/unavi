#[allow(warnings)]
mod bindings;

struct Layout;

bindings::export!(Layout with_types_in bindings);
