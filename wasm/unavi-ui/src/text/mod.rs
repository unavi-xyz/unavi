use crate::{bindings::exports::unavi::ui::text::Guest, GuestImpl};

mod text;
mod text_box;

impl Guest for GuestImpl {
    type Text = text::Text;
    type TextBox = text_box::TextBox;
}
