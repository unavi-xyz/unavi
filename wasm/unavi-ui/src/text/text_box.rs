use std::cell::Cell;

use crate::bindings::{
    exports::unavi::ui::text::{GuestText, GuestTextBox, Text as TextExport, WordWrap},
    unavi::layout::container::{Alignment, Container},
};

use super::text::Text;

pub struct TextBox {
    root: Container,
    text: Text,
    wrap: Cell<WordWrap>,
}

impl TextBox {
    fn generate(&self) {
        let size = self.root.size();

        let mut text = self.text.text();

        for _ in 0..50 {
            let mut lines = Vec::default();
            let mut needs_update = false;

            for (line, data) in self.text.generate_mesh_data(text) {
                let Ok(data) = data else { continue };

                // Word wrap when overflow x.
                let len_x = data.bbox.max.x - data.bbox.min.x;
                if len_x > size.x {
                    // Try to guess correct place to split line.
                    let percent = size.x / len_x;
                    let mut idx = (line.len() as f32 * percent).floor() as usize;

                    // Check guess, walking backwards through characters until it fits.
                    while idx > 0 {
                        let split = line.split_at(idx);

                        let is_valid_split = match self.wrap.get() {
                            WordWrap::Character => true,
                            WordWrap::Word => split.0.ends_with(' ') || split.1.starts_with(' '),
                        };

                        if !is_valid_split {
                            idx -= 1;
                            continue;
                        }

                        if let Some(data) = &self
                            .text
                            .generate_mesh_data(split.0.to_owned())
                            .into_iter()
                            .flat_map(|(_, d)| d)
                            .next()
                        {
                            let new_len_x = data.bbox.max.x - data.bbox.min.x;
                            if new_len_x > size.x {
                                idx -= 1;
                            } else {
                                break;
                            }
                        } else {
                            break;
                        };
                    }

                    if idx == 0 {
                        // TODO: Reduce font size
                        break;
                    }

                    let split = line.split_at(idx);
                    lines.push(split.0.to_owned());
                    lines.push(split.1.to_owned());
                    needs_update = true;
                    break;
                }

                // TODO: Reduce font size when overflow y.

                lines.push(line);
            }

            text = lines.join("\n");

            if !needs_update {
                break;
            }
        }

        self.root.set_align_x(self.text.alignment());
        self.text.set_text(text);
    }
}

impl GuestTextBox for TextBox {
    fn new(root: Container) -> Self {
        let text = Text::new(String::default());
        text.set_alignment(Alignment::Start);
        root.inner().set_mesh(Some(&text.mesh()));

        Self {
            root,
            text,
            wrap: Cell::new(WordWrap::Word),
        }
    }

    fn root(&self) -> Container {
        self.root.ref_()
    }
    fn text(&self) -> TextExport {
        self.text.ref_()
    }

    fn set_text(&self, value: String) {
        self.text.set_text(value);
        self.generate();
    }

    fn wrap(&self) -> WordWrap {
        self.wrap.get()
    }
    fn set_wrap(&self, value: WordWrap) {
        self.wrap.set(value);
    }
}
