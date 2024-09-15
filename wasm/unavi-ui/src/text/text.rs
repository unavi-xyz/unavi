use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use meshtext::{error::MeshTextError, IndexedMeshText, MeshGenerator, OwnedFace, TextSection};

use crate::bindings::{
    exports::unavi::ui::text::{GuestText, Text as TextExport},
    wired::scene::mesh::Mesh,
};

const FONT: &[u8] = include_bytes!("../../Roboto-Regular.ttf");

#[derive(Clone)]
pub struct Text(Rc<TextData>);

struct TextData {
    font_size: Cell<f32>,
    generator: RefCell<MeshGenerator<OwnedFace>>,
    mesh: Mesh,
    text: RefCell<String>,
    thickness: Cell<f32>,
}

pub type GeneratedMesh = Result<IndexedMeshText, Box<dyn MeshTextError>>;

impl Text {
    pub fn generate_mesh_data(&self, text: String) -> Vec<(String, GeneratedMesh)> {
        let font_size = self.0.font_size.get();
        let thickness = self.0.thickness.get();

        let flat = thickness == 0.0;
        let transform = [
            font_size, 0.0, 0.0, 0.0, 0.0, font_size, 0.0, 0.0, 0.0, 0.0, thickness, 0.0, 0.0, 0.0,
            0.0, 1.0,
        ];

        text.split('\n')
            .map(|line| {
                (
                    line.to_owned(),
                    self.0
                        .generator
                        .borrow_mut()
                        .generate_section(line, flat, Some(&transform)),
                )
            })
            .collect()
    }

    fn generate(&self) {
        for primitive in self.0.mesh.list_primitives() {
            self.0.mesh.remove_primitive(primitive);
        }

        let line_padding = 1.25;
        let mut total_line_height = 0.0;

        for (i, mut data) in self
            .generate_mesh_data(self.0.text.borrow().clone())
            .into_iter()
            .flat_map(|(_, d)| d)
            .enumerate()
        {
            let primitive = self.0.mesh.create_primitive();

            let line_height = (data.bbox.max.y - data.bbox.min.y).abs();
            let offset_factor = (if i == 0 { 1.0 / line_padding } else { 0.5 }) * line_padding;
            let line_offset = line_height * offset_factor;

            data.vertices = data
                .vertices
                .chunks(3)
                .flat_map(|v| [v[0], v[1] - total_line_height - line_offset, v[2]])
                .collect();

            primitive.set_indices(&data.indices);
            primitive.set_positions(&data.vertices);

            total_line_height += if i == 0 {
                line_height * line_padding * 1.5
            } else {
                line_height * line_padding
            };
        }
    }
}

impl GuestText for Text {
    fn new(text: String) -> Self {
        let mesh = Mesh::new();

        let text = Self(Rc::new(TextData {
            font_size: Cell::new(0.25),
            generator: RefCell::new(MeshGenerator::new(FONT.to_owned())),
            mesh,
            text: RefCell::new(text),
            thickness: Cell::default(),
        }));

        text.generate();

        text
    }

    fn ref_(&self) -> TextExport {
        TextExport::new(self.clone())
    }

    fn set_font(&self, value: Option<Vec<u8>>) {
        self.0
            .generator
            .replace(MeshGenerator::new(value.unwrap_or(FONT.to_owned())));
        self.generate();
    }

    fn text(&self) -> String {
        self.0.text.borrow().clone()
    }
    fn set_text(&self, value: String) {
        if self.0.text.borrow().as_str() == value {
            return;
        }

        self.0.text.replace(value);
        self.generate();
    }

    fn font_size(&self) -> f32 {
        self.0.font_size.get()
    }
    fn set_font_size(&self, value: f32) {
        self.0.font_size.set(value);
        self.generate();
    }

    fn thickness(&self) -> f32 {
        self.0.thickness.get()
    }
    fn set_thickness(&self, value: f32) {
        self.0.thickness.set(value);
        self.generate();
    }

    fn mesh(&self) -> Mesh {
        self.0.mesh.ref_()
    }
}
