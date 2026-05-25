use std::{
    cell::{
        Cell,
        RefCell,
    },
    f32::consts::TAU,
};

use crate::{
    RawMesh,
    exports::unavi::shapes::api::GuestCone,
    wired::scene::types::{
        Document,
        Prim,
    },
};

#[derive(Default)]
pub struct ConeWrapped {
    doc:        RefCell<Option<Document>>,
    radius:     f32,
    height:     f32,
    resolution: Cell<u32>,
}

impl GuestCone for ConeWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            radius,
            height,
            resolution: Cell::new(32),
            ..Default::default()
        }
    }

    fn mesh(&self) -> Prim {
        crate::convert_raw_mesh(
            self.doc.borrow().as_ref(),
            build(self.radius, self.height, self.resolution.get()),
        )
    }

    fn set_doc(&self, doc: Document) {
        *self.doc.borrow_mut() = Some(doc);
    }

    fn resolution(&self) -> u32 {
        self.resolution.get()
    }

    fn set_resolution(&self, value: u32) {
        self.resolution.set(value);
    }
}

fn build(radius: f32, height: f32, res: u32) -> RawMesh {
    let res = res.max(3) as usize;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    let half_h = height / 2.0;

    // Base cap (y = -half_h, normal = -Y)
    let base_center = positions.len() as u32;
    positions.push([0.0, -half_h, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    let base_ring = positions.len() as u32;
    for i in 0..res {
        let a = TAU * i as f32 / res as f32;
        let (s, c) = a.sin_cos();
        positions.push([radius * c, -half_h, radius * s]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([0.5f32.mul_add(c, 0.5), 0.5f32.mul_add(-s, 0.5)]);
    }
    for i in 0..res {
        let next = (i + 1) % res;
        indices.extend_from_slice(&[base_center, base_ring + i as u32, base_ring + next as u32]);
    }

    // Side: slanted normals
    let slope_len = radius.hypot(height);
    let ny = radius / slope_len;
    let nxz = height / slope_len;

    let side_base = positions.len() as u32;
    // Base ring verts for the side (duplicated for different normals)
    for i in 0..=res {
        let a = TAU * i as f32 / res as f32;
        let (s, c) = a.sin_cos();
        positions.push([radius * c, -half_h, radius * s]);
        normals.push([nxz * c, ny, nxz * s]);
        uvs.push([i as f32 / res as f32, 1.0]);
    }
    // Apex — duplicated per segment for sharp tip normals
    let apex_base = positions.len() as u32;
    for i in 0..=res {
        let a = TAU * (i as f32 + 0.5) / res as f32;
        let (s, c) = a.sin_cos();
        positions.push([0.0, half_h, 0.0]);
        normals.push([nxz * c, ny, nxz * s]);
        uvs.push([(i as f32 + 0.5) / res as f32, 0.0]);
    }
    for i in 0..res as u32 {
        indices.extend_from_slice(&[side_base + i, apex_base + i, side_base + i + 1]);
    }

    RawMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}
