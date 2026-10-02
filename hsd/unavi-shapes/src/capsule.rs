//! A cylinder capped with two hemispheres.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    f32::consts::{
        FRAC_PI_2,
        TAU,
    },
};

use crate::{
    RawMesh,
    exports::unavi::shapes::api::GuestCapsule,
    wired::{
        core::error::Error,
        scene::{
            document::Document,
            properties::{
                Collider,
                ColliderCapsule,
            },
        },
    },
};

const MAX_LATITUDES: u32 = 64;
const MAX_LONGITUDES: u32 = 128;
const MAX_RINGS: u32 = 32;

pub struct Capsule {
    doc:         RefCell<Option<Document>>,
    radius:      f32,
    half_length: f32,
    latitudes:   Cell<u32>,
    longitudes:  Cell<u32>,
    rings:       Cell<u32>,
}

impl GuestCapsule for Capsule {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            doc: RefCell::new(None),
            radius,
            half_length: height * 0.5,
            latitudes: Cell::new(16),
            longitudes: Cell::new(32),
            rings: Cell::new(0),
        }
    }

    fn collider(&self) -> Collider {
        Collider::Capsule(ColliderCapsule {
            height: self.half_length * 2.0,
            radius: self.radius,
        })
    }

    fn mesh(&self) -> Result<(u64, u64), Error> {
        crate::mesh_into(
            &self.doc,
            &build(
                self.radius,
                self.half_length,
                self.latitudes.get(),
                self.longitudes.get(),
                self.rings.get(),
            ),
        )
    }

    fn set_doc(&self, doc: Document) {
        *self.doc.borrow_mut() = Some(doc);
    }

    fn latitudes(&self) -> u32 {
        self.latitudes.get()
    }

    fn set_latitudes(&self, value: u32) -> Result<(), Error> {
        if value > MAX_LATITUDES {
            return Err(Error::InvalidArgument(format!(
                "latitudes must be at most {MAX_LATITUDES}"
            )));
        }
        self.latitudes.set(value);
        Ok(())
    }

    fn longitudes(&self) -> u32 {
        self.longitudes.get()
    }

    fn set_longitudes(&self, value: u32) -> Result<(), Error> {
        if value > MAX_LONGITUDES {
            return Err(Error::InvalidArgument(format!(
                "longitudes must be at most {MAX_LONGITUDES}"
            )));
        }
        self.longitudes.set(value);
        Ok(())
    }

    fn rings(&self) -> u32 {
        self.rings.get()
    }

    fn set_rings(&self, value: u32) -> Result<(), Error> {
        if value > MAX_RINGS {
            return Err(Error::InvalidArgument(format!(
                "rings must be at most {MAX_RINGS}"
            )));
        }
        self.rings.set(value);
        Ok(())
    }
}

#[expect(clippy::many_single_char_names)]
fn build(radius: f32, half_len: f32, latitudes: u32, longitudes: u32, rings: u32) -> RawMesh {
    let lats = (latitudes.max(4) & !1) as usize; // ensure even
    let lons = longitudes.max(3) as usize;
    let rings = rings as usize;

    let hemi_lats = lats / 2;
    // total rings: hemi_lats (bottom) + rings+1 (cylinder) + hemi_lats (top)
    let total_rings = hemi_lats + rings + 1 + hemi_lats;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for k in 0..=total_rings {
        let (y, xz_r, ny, nxz) = ring_params(k, hemi_lats, rings, radius, half_len);
        let v = k as f32 / total_rings as f32;
        for i in 0..=lons {
            let a = TAU * i as f32 / lons as f32;
            let (s, c) = a.sin_cos();
            positions.push([xz_r * c, y, xz_r * s]);
            normals.push([nxz * c, ny, nxz * s]);
            uvs.push([i as f32 / lons as f32, v]);
        }
    }

    let row = (lons + 1) as u32;
    for k in 0..total_rings as u32 {
        for i in 0..lons as u32 {
            let a = k * row + i;
            let b = a + 1;
            let c = a + row;
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

/// Returns (y, `xz_radius`, `normal_y`, `normal_xz`) for the k-th ring.
fn ring_params(
    k: usize,
    hemi_lats: usize,
    rings: usize,
    radius: f32,
    half_len: f32,
) -> (f32, f32, f32, f32) {
    if k <= hemi_lats {
        let t = FRAC_PI_2 * (k as f32 / hemi_lats as f32 - 1.0);
        let (st, ct) = t.sin_cos();
        (radius.mul_add(st, -half_len), radius * ct, st, ct)
    } else if k <= hemi_lats + rings + 1 {
        let seg = k - hemi_lats;
        let y = -half_len + 2.0 * half_len * seg as f32 / (rings + 1) as f32;
        (y, radius, 0.0, 1.0)
    } else {
        let seg = k - (hemi_lats + rings + 1);
        let t = FRAC_PI_2 * seg as f32 / hemi_lats as f32;
        let (st, ct) = t.sin_cos();
        (radius.mul_add(st, half_len), radius * ct, st, ct)
    }
}
