use std::cell::Cell;

use meshtext::{IndexedMeshText, MeshGenerator, TextSection};

use crate::{
    bindings::{
        exports::unavi::ui::text::{Guest, GuestText, Node},
        wired::scene::mesh::Mesh,
    },
    GuestImpl,
};

const FONT: &[u8] = include_bytes!("../Roboto-Regular.ttf");

impl Guest for GuestImpl {
    type Text = Text;
}

pub struct Text {
    flat: Cell<bool>,
    text: String,
}

impl GuestText for Text {
    fn new(text: String) -> Self {
        Self {
            flat: Cell::new(true),
            text,
        }
    }

    fn flat(&self) -> bool {
        self.flat.get()
    }
    fn set_flat(&self, value: bool) {
        self.flat.set(value);
    }

    fn to_mesh(&self) -> Option<Mesh> {
        let mut generator = MeshGenerator::new(FONT);

        let data: IndexedMeshText = generator
            .generate_section(&self.text, self.flat.get(), None)
            .ok()?;

        let mesh = Mesh::new();
        let primitive = mesh.create_primitive();

        primitive.set_indices(&data.indices);
        primitive.set_positions(&data.vertices);

        Some(mesh)
    }
    fn to_node(&self) -> Option<Node> {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()?));
        Some(node)
    }
}
