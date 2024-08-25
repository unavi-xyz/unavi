#[allow(warnings)]
mod bindings;

mod button;
mod text;

struct GuestImpl;

bindings::export!(GuestImpl with_types_in bindings);
