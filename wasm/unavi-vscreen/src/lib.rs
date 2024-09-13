use bindings::exports::unavi::vscreen::screen::Guest;

#[allow(warnings)]
mod bindings;
mod screen;
mod wired_input_impls;
mod wired_math_impls;

struct Types;

impl Guest for Types {
    type Screen = screen::Screen;
}

bindings::export!(Types with_types_in bindings);
