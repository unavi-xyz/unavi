use bindings::exports::unavi::vscreen::screen::Guest;

#[allow(warnings)]
mod bindings;
mod module;
mod screen;

struct Types;

impl Guest for Types {
    type Module = module::Module;
    type Screen = screen::Screen;
}

bindings::export!(Types with_types_in bindings);
