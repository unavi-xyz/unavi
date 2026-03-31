use std::cell::Cell;
use std::f32::consts::TAU;

use crate::{
    RawMesh,
    exports::unavi::shapes::api::GuestCylinder,
    wired::scene::types::{Collider, ColliderCylinder, Mesh},
};

pub struct CylinderWrapped {
    radius: f32,
    half_height: f32,
    resolution: Cell<u32>,
    segments: Cell<u32>,
}

impl GuestCylinder for CylinderWrapped {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            radius,
            half_height: height * 0.5,
            resolution: Cell::new(32),
            segments: Cell::new(1),
        }
    }

    fn collider(&self) -> Collider {
        Collider::Cylinder(ColliderCylinder {
            height: self.half_height * 2.0,
            radius: self.radius,
        })
    }

    fn mesh(&self) -> Mesh {
        crate::convert_raw_mesh(build(
            self.radius,
            self.half_height,
            self.resolution.get(),
            self.segments.get(),
        ))
    }

    fn resolution(&self) -> u32 {
        self.resolution.get()
    }

    fn set_resolution(&self, value: u32) {
        self.resolution.set(value);
    }

    fn segments(&self) -> u32 {
        self.segments.get()
    }

    fn set_segments(&self, value: u32) {
        self.segments.set(value);
    }
}

#[expect(clippy::many_single_char_names)]
fn build(radius: f32, half_h: f32, res: u32, segs: u32) -> RawMesh {
    let res = res.max(3) as usize;
    let segs = segs.max(1) as usize;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Top cap
    let top_center = positions.len() as u32;
    positions.push([0.0, half_h, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    let top_ring_start = positions.len() as u32;
    for i in 0..res {
        let a = TAU * i as f32 / res as f32;
        let (s, c) = a.sin_cos();
        positions.push([radius * c, half_h, radius * s]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.5f32.mul_add(c, 0.5), 0.5f32.mul_add(s, 0.5)]);
    }
    for i in 0..res {
        let next = (i + 1) % res;
        indices.extend_from_slice(&[
            top_center,
            top_ring_start + next as u32,
            top_ring_start + i as u32,
        ]);
    }

    // Bottom cap
    let bot_center = positions.len() as u32;
    positions.push([0.0, -half_h, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    let bot_ring_start = positions.len() as u32;
    for i in 0..res {
        let a = TAU * i as f32 / res as f32;
        let (s, c) = a.sin_cos();
        positions.push([radius * c, -half_h, radius * s]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push([0.5f32.mul_add(c, 0.5), 0.5f32.mul_add(-s, 0.5)]);
    }
    for i in 0..res {
        let next = (i + 1) % res;
        indices.extend_from_slice(&[
            bot_center,
            bot_ring_start + i as u32,
            bot_ring_start + next as u32,
        ]);
    }

    // Side rings: segs+1 rings from bottom to top
    let side_start = positions.len() as u32;
    for seg in 0..=segs {
        let y = -half_h + 2.0 * half_h * seg as f32 / segs as f32;
        let v = seg as f32 / segs as f32;
        for i in 0..=res {
            let a = TAU * i as f32 / res as f32;
            let (s, c) = a.sin_cos();
            positions.push([radius * c, y, radius * s]);
            normals.push([c, 0.0, s]);
            uvs.push([i as f32 / res as f32, v]);
        }
    }
    let ring_verts = (res + 1) as u32;
    for seg in 0..segs as u32 {
        for i in 0..res as u32 {
            let a = side_start + seg * ring_verts + i;
            let b = a + 1;
            let c = a + ring_verts;
            let d = c + 1;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    RawMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}
