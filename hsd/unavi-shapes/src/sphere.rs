//! A subdivided icosahedron, projected onto a sphere.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    collections::HashMap,
};

use wired_guest::math::Vec3;

use crate::{
    RawMesh,
    exports::unavi::shapes::api::GuestSphere,
    wired::{
        core::error::Error,
        scene::{
            document::Document,
            properties::Collider,
        },
    },
};

/// `20 * 4^subdivisions` triangles, already 81,920 at the limit.
const MAX_SUBDIVISIONS: u32 = 6;

pub struct Sphere {
    doc:          RefCell<Option<Document>>,
    radius:       f32,
    subdivisions: Cell<u32>,
}

impl GuestSphere for Sphere {
    fn new(radius: f32) -> Self {
        Self {
            doc: RefCell::new(None),
            radius,
            subdivisions: Cell::new(5),
        }
    }

    fn collider(&self) -> Collider {
        Collider::Sphere(self.radius)
    }

    fn mesh(&self) -> Result<(u64, u64), Error> {
        crate::mesh_into(&self.doc, &build(self.radius, self.subdivisions.get()))
    }

    fn set_doc(&self, doc: Document) {
        *self.doc.borrow_mut() = Some(doc);
    }

    fn subdivisions(&self) -> u32 {
        self.subdivisions.get()
    }

    fn set_subdivisions(&self, value: u32) -> Result<(), Error> {
        if value > MAX_SUBDIVISIONS {
            return Err(Error::InvalidArgument(format!(
                "subdivisions must be at most {MAX_SUBDIVISIONS}"
            )));
        }
        self.subdivisions.set(value);
        Ok(())
    }
}

fn midpoint(a: u32, b: u32, verts: &mut Vec<Vec3>, cache: &mut HashMap<(u32, u32), u32>) -> u32 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&i) = cache.get(&key) {
        return i;
    }
    let mid = (verts[a as usize] + verts[b as usize]).normalize();
    let i = verts.len() as u32;
    verts.push(mid);
    cache.insert(key, i);
    i
}

fn build(radius: f32, subdivisions: u32) -> RawMesh {
    // Start with a regular icosahedron
    let t = f32::midpoint(1.0, 5.0_f32.sqrt());
    let base_verts: &[[f32; 3]] = &[
        [-1.0, t, 0.0],
        [1.0, t, 0.0],
        [-1.0, -t, 0.0],
        [1.0, -t, 0.0],
        [0.0, -1.0, t],
        [0.0, 1.0, t],
        [0.0, -1.0, -t],
        [0.0, 1.0, -t],
        [t, 0.0, -1.0],
        [t, 0.0, 1.0],
        [-t, 0.0, -1.0],
        [-t, 0.0, 1.0],
    ];

    let mut verts: Vec<Vec3> = base_verts
        .iter()
        .map(|v| Vec3::from(*v).normalize())
        .collect();

    let base_tris: &[[u32; 3]] = &[
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];

    let mut tris: Vec<[u32; 3]> = base_tris.to_vec();

    let mut cache: HashMap<(u32, u32), u32> = HashMap::new();

    for _ in 0..subdivisions {
        let mut new_tris = Vec::with_capacity(tris.len() * 4);
        for [a, b, c] in &tris {
            let ab = midpoint(*a, *b, &mut verts, &mut cache);
            let bc = midpoint(*b, *c, &mut verts, &mut cache);
            let ca = midpoint(*c, *a, &mut verts, &mut cache);
            new_tris.push([*a, ab, ca]);
            new_tris.push([*b, bc, ab]);
            new_tris.push([*c, ca, bc]);
            new_tris.push([ab, bc, ca]);
        }
        tris = new_tris;
        cache.clear();
    }

    let positions: Vec<[f32; 3]> = verts.iter().map(|v| (*v * radius).into()).collect();
    let normals: Vec<[f32; 3]> = verts.iter().map(|v| (*v).into()).collect();
    let uvs: Vec<[f32; 2]> = verts
        .iter()
        .map(|v| {
            let u = 0.5 + v.z.atan2(v.x) / std::f32::consts::TAU;
            let vv = 0.5 - v.y.asin() / std::f32::consts::PI;
            [u, vv]
        })
        .collect();
    let indices: Vec<u32> = tris.iter().flat_map(|t| t.iter().copied()).collect();

    RawMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}
