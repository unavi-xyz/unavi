use std::{
    cell::{
        Cell,
        RefCell,
    },
    f32::consts::TAU,
};

use glam::Vec3;

use crate::{
    RawMesh,
    exports::unavi::shapes::api::GuestTorus,
    wired::scene::types::{
        Document,
        Prim,
    },
};

#[derive(Default)]
pub struct TorusWrapped {
    doc:              RefCell<Option<Document>>,
    minor_radius:     f32,
    major_radius:     f32,
    minor_resolution: Cell<u32>,
    major_resolution: Cell<u32>,
}

impl GuestTorus for TorusWrapped {
    fn new(minor_radius: f32, major_radius: f32) -> Self {
        Self {
            minor_radius,
            major_radius,
            minor_resolution: Cell::new(24),
            major_resolution: Cell::new(32),
            ..Default::default()
        }
    }

    fn mesh(&self) -> Prim {
        crate::convert_raw_mesh(
            self.doc.borrow().as_ref(),
            build(
                self.minor_radius,
                self.major_radius,
                self.minor_resolution.get() as usize,
                self.major_resolution.get() as usize,
            ),
        )
    }

    fn set_doc(&self, doc: Document) {
        *self.doc.borrow_mut() = Some(doc);
    }

    fn minor_resolution(&self) -> u32 {
        self.minor_resolution.get()
    }

    fn set_minor_resolution(&self, value: u32) {
        self.minor_resolution.set(value);
    }

    fn major_resolution(&self) -> u32 {
        self.major_resolution.get()
    }

    fn set_major_resolution(&self, value: u32) {
        self.major_resolution.set(value);
    }
}

#[expect(clippy::many_single_char_names)]
fn build(r: f32, big_r: f32, minor_res: usize, major_res: usize) -> RawMesh {
    let minor_res = minor_res.max(3);
    let major_res = major_res.max(3);

    let total_verts = major_res * minor_res;
    let mut positions = Vec::with_capacity(total_verts);
    let mut normals = Vec::with_capacity(total_verts);
    let mut uvs = Vec::with_capacity(total_verts);
    let mut indices = Vec::with_capacity(major_res * minor_res * 6);

    for i in 0..major_res {
        let theta = TAU * i as f32 / major_res as f32;
        let (st, ct) = theta.sin_cos();
        // Center of the minor circle at this theta
        let center = Vec3::new(big_r * ct, 0.0, big_r * st);

        for j in 0..minor_res {
            let phi = TAU * j as f32 / minor_res as f32;
            let (sp, cp) = phi.sin_cos();

            let pos = Vec3::new(r.mul_add(cp, big_r) * ct, r * sp, r.mul_add(cp, big_r) * st);
            let normal = (pos - center).normalize();

            positions.push(pos.into());
            normals.push(normal.into());
            uvs.push([i as f32 / major_res as f32, j as f32 / minor_res as f32]);
        }
    }

    for i in 0..major_res {
        let i_next = (i + 1) % major_res;
        for j in 0..minor_res {
            let j_next = (j + 1) % minor_res;
            let a = (i * minor_res + j) as u32;
            let b = (i * minor_res + j_next) as u32;
            let c = (i_next * minor_res + j) as u32;
            let d = (i_next * minor_res + j_next) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }

    RawMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}
