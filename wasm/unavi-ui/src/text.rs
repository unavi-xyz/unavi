use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use meshtext::{IndexedMeshText, MeshGenerator, TextSection};

use crate::{
    bindings::{
        exports::unavi::ui::text::{Guest, GuestText, GuestTextBox, Text as TextExport},
        unavi::layout::container::Container,
        wired::scene::mesh::{Mesh, Primitive},
    },
    GuestImpl,
};

const FONT: &[u8] = include_bytes!("../Roboto-Regular.ttf");

impl Guest for GuestImpl {
    type Text = Text;
    type TextBox = TextBox;
}

#[derive(Clone)]
pub struct Text(Rc<TextData>);

struct TextData {
    flat: Cell<bool>,
    mesh: Mesh,
    primitive: Primitive,
    text: RefCell<String>,
}

impl GuestText for Text {
    fn new(text: String) -> Self {
        let mesh = Mesh::new();
        let primitive = mesh.create_primitive();

        Self(Rc::new(TextData {
            flat: Cell::new(true),
            mesh,
            primitive,
            text: RefCell::new(text),
        }))
    }

    fn ref_(&self) -> TextExport {
        TextExport::new(self.clone())
    }

    fn text(&self) -> String {
        self.0.text.borrow().clone()
    }
    fn set_text(&self, value: String) {
        self.0.text.replace(value);

        let mut generator = MeshGenerator::new(FONT);

        let data: Result<IndexedMeshText, _> =
            generator.generate_section(&self.0.text.borrow(), self.0.flat.get(), None);

        if let Ok(data) = data {
            self.0.primitive.set_indices(&data.indices);
            self.0.primitive.set_positions(&data.vertices);
        } else {
            self.0.primitive.set_positions(&[]);
            self.0.primitive.set_indices(&[]);
        }
    }

    fn flat(&self) -> bool {
        self.0.flat.get()
    }
    fn set_flat(&self, value: bool) {
        self.0.flat.set(value);
    }

    fn mesh(&self) -> Mesh {
        self.0.mesh.ref_()
    }
}

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
