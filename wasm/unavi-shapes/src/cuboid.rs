use std::cell::RefCell;

use glam::Vec3;
use wired_prelude::wired_math::types::Vec3 as WVec3;

use crate::{
    RawMesh,
    exports::unavi::shapes::api::GuestCuboid,
    wired::scene::types::{
        Collider,
        Document,
        Prim,
    },
};

#[derive(Default)]
pub struct CuboidWrapped {
    doc:  RefCell<Option<Document>>,
    half: Vec3,
}

impl GuestCuboid for CuboidWrapped {
    fn new(size: WVec3) -> Self {
        Self {
            half: Vec3::new(size.x * 0.5, size.y * 0.5, size.z * 0.5),
            ..Default::default()
        }
    }

    fn collider(&self) -> Collider {
        Collider::Cuboid(WVec3::new(
            self.half.x * 2.0,
            self.half.y * 2.0,
            self.half.z * 2.0,
        ))
    }

    fn mesh(&self) -> Prim {
        crate::convert_raw_mesh(self.doc.borrow().as_ref(), build(self.half))
    }

    fn set_doc(&self, doc: Document) {
        *self.doc.borrow_mut() = Some(doc);
    }
}

fn build(h: Vec3) -> RawMesh {
    // 6 faces × 4 verts = 24 verts, 6 faces × 6 indices = 36 indices
    let faces: [([f32; 3], [[f32; 3]; 4], [[f32; 2]; 4]); 6] = [
        (
            [1.0, 0.0, 0.0],
            [
                [h.x, -h.y, -h.z],
                [h.x, h.y, -h.z],
                [h.x, h.y, h.z],
                [h.x, -h.y, h.z],
            ],
            [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        ),
        (
            [-1.0, 0.0, 0.0],
            [
                [-h.x, -h.y, h.z],
                [-h.x, h.y, h.z],
                [-h.x, h.y, -h.z],
                [-h.x, -h.y, -h.z],
            ],
            [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        ),
        (
            [0.0, 1.0, 0.0],
            [
                [-h.x, h.y, -h.z],
                [-h.x, h.y, h.z],
                [h.x, h.y, h.z],
                [h.x, h.y, -h.z],
            ],
            [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        ),
        (
            [0.0, -1.0, 0.0],
            [
                [-h.x, -h.y, h.z],
                [-h.x, -h.y, -h.z],
                [h.x, -h.y, -h.z],
                [h.x, -h.y, h.z],
            ],
            [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        ),
        (
            [0.0, 0.0, 1.0],
            [
                [-h.x, -h.y, h.z],
                [h.x, -h.y, h.z],
                [h.x, h.y, h.z],
                [-h.x, h.y, h.z],
            ],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
        (
            [0.0, 0.0, -1.0],
            [
                [h.x, -h.y, -h.z],
                [-h.x, -h.y, -h.z],
                [-h.x, h.y, -h.z],
                [h.x, h.y, -h.z],
            ],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        ),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (i, (normal, verts, face_uvs)) in faces.iter().enumerate() {
        let base = (i * 4) as u32;
        for j in 0..4 {
            positions.push(verts[j]);
            normals.push(*normal);
            uvs.push(face_uvs[j]);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    RawMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}
