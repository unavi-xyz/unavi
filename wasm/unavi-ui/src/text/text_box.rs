use crate::bindings::{
    exports::unavi::ui::text::{GuestText, GuestTextBox, Text as TextExport},
    unavi::layout::container::Container,
};

use super::text::Text;

pub struct TextBox {
    root: Container,
    text: Text,
}

impl GuestTextBox for TextBox {
    fn new(root: Container) -> Self {
        let text = Text::new(String::default());
        root.inner().set_mesh(Some(&text.mesh()));

        Self { root, text }
    }

    fn root(&self) -> Container {
        self.root.ref_()
    }
    fn text(&self) -> TextExport {
        self.text.ref_()
    }
}
