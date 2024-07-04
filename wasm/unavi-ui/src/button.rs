use crate::{
    bindings::exports::unavi::ui::{
        button::{Guest, GuestButton},
        container::GuestContainer,
    },
    GuestImpl,
};

use super::container::Container;

impl Guest for GuestImpl {
    type Button = Button;
}

pub struct Button {
    root: Container,
}

impl GuestButton for Button {
    fn new() -> Self {
        Self {
            root: Container::new(),
        }
    }

    fn root(&self) -> crate::bindings::exports::unavi::ui::button::Container {
        todo!();
    }
}
