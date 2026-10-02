//! A box, built from six quads.

use std::cell::RefCell;

use wired_guest::math::Vec3;

use crate::{
    RawMesh,
    exports::unavi::shapes::api::GuestCuboid,
    wired::{
        core::error::Error,
        scene::{
            document::Document,
            properties::Collider,
        },
    },
};

/// `size` is the full extent along each axis.
pub struct Cuboid {
    doc:  RefCell<Option<Document>>,
    size: Vec3,
}

impl GuestCuboid for Cuboid {
    fn new(size: Vec3) -> Self {
        Self {
            doc: RefCell::new(None),
            size,
        }
    }

    fn collider(&self) -> Collider {
        Collider::Cuboid(self.size)
    }

    fn mesh(&self) -> Result<(u64, u64), Error> {
        crate::mesh_into(&self.doc, &build(self.size * 0.5))
    }

    fn set_doc(&self, doc: Document) {
        *self.doc.borrow_mut() = Some(doc);
    }
}

/// One flat face of the cuboid: its outward normal, its four corners wound
/// counter-clockwise, and the UV at each corner.
struct Face {
    normal: [f32; 3],
    verts:  [[f32; 3]; 4],
    uvs:    [[f32; 2]; 4],
}

fn build(h: Vec3) -> RawMesh {
    let faces = [
        Face {
            normal: [1.0, 0.0, 0.0],
            verts:  [
                [h.x, -h.y, -h.z],
                [h.x, h.y, -h.z],
                [h.x, h.y, h.z],
                [h.x, -h.y, h.z],
            ],
            uvs:    [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        },
        Face {
            normal: [-1.0, 0.0, 0.0],
            verts:  [
                [-h.x, -h.y, h.z],
                [-h.x, h.y, h.z],
                [-h.x, h.y, -h.z],
                [-h.x, -h.y, -h.z],
            ],
            uvs:    [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        },
        Face {
            normal: [0.0, 1.0, 0.0],
            verts:  [
                [-h.x, h.y, -h.z],
                [-h.x, h.y, h.z],
                [h.x, h.y, h.z],
                [h.x, h.y, -h.z],
            ],
            uvs:    [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        },
        Face {
            normal: [0.0, -1.0, 0.0],
            verts:  [
                [-h.x, -h.y, h.z],
                [-h.x, -h.y, -h.z],
                [h.x, -h.y, -h.z],
                [h.x, -h.y, h.z],
            ],
            uvs:    [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        },
        Face {
            normal: [0.0, 0.0, 1.0],
            verts:  [
                [-h.x, -h.y, h.z],
                [h.x, -h.y, h.z],
                [h.x, h.y, h.z],
                [-h.x, h.y, h.z],
            ],
            uvs:    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        },
        Face {
            normal: [0.0, 0.0, -1.0],
            verts:  [
                [h.x, -h.y, -h.z],
                [-h.x, -h.y, -h.z],
                [-h.x, h.y, -h.z],
                [h.x, h.y, -h.z],
            ],
            uvs:    [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
        },
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (i, face) in faces.iter().enumerate() {
        let base = (i * 4) as u32;
        for j in 0..4 {
            positions.push(face.verts[j]);
            normals.push(face.normal);
            uvs.push(face.uvs[j]);
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
