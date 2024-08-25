use crate::{
    bindings::{
        exports::unavi::ui::button::{Guest, GuestButton},
        unavi::layout::container::Container,
    },
    GuestImpl,
};

impl Guest for GuestImpl {
    type Button = Button;
}

pub struct Button {
    root: Container,
}

impl GuestButton for Button {
    fn new(root: Container) -> Self {
        Self { root }
    }

    fn root(&self) -> crate::bindings::exports::unavi::ui::button::Container {
        todo!()
    }
}
