//! Where dragged motes end up: ordinary dynamic bodies, grabbable like
//! anything else in the room.

use std::cell::Cell;

use unavi_vui::{
    mesh,
    view::Style,
};
use wired_prelude::prelude::*;

use crate::wired::{
    physics::api::set_linear_velocity,
    scene::types::{
        AlphaMode,
        Collider,
        Document,
        Material,
        Prim,
        RigidBody,
        RigidBodyKind,
        Xform,
    },
};

/// One size for every planted body, so the mesh and collider are built once
/// at load and a plant costs no uploads at all.
const RADIUS: f32 = 0.05;
const SPHERE_RINGS: usize = 10;
const SPHERE_SEGMENTS: usize = 16;

pub struct Planted {
    bodies: Vec<Prim>,
    next:   Cell<usize>,
    used:   Cell<usize>,
}

impl Planted {
    pub fn new(doc: &Document, capacity: usize) -> anyhow::Result<Self> {
        let sphere = mesh::sphere(RADIUS, SPHERE_RINGS, SPHERE_SEGMENTS);
        let mut bodies = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            let prim = doc.create_prim()?;
            prim.set_mesh_stream("POSITION", Some(&sphere.positions))?;
            prim.set_mesh_stream("NORMAL", Some(&sphere.normals))?;
            prim.set_mesh_indices_u32(Some(&sphere.indices))?;
            prim.set_xform(Some(hidden()))?;
            bodies.push(prim);
        }

        Ok(Self {
            bodies,
            next: Cell::new(0),
            used: Cell::new(0),
        })
    }

    /// Drops a body at `at`, carrying the throw's momentum.
    pub fn plant(&self, at: Vec3, velocity: Vec3, style: Style) -> anyhow::Result<bool> {
        let Some(prim) = self.bodies.get(self.next.get()) else {
            return Ok(false);
        };
        let recycled = self.used.get() >= self.bodies.len();
        self.next.set((self.next.get() + 1) % self.bodies.len());
        self.used.set(self.used.get() + 1);

        prim.set_material(Some(material(style)))?;
        prim.set_xform(Some(Xform {
            translation: at,
            rotation:    Quat::IDENTITY,
            scale:       Vec3::ONE,
        }))?;
        prim.set_collider(Some(Collider::Sphere(RADIUS)))?;
        prim.set_rigid_body(Some(RigidBody {
            kind:            RigidBodyKind::Dynamic,
            angular_damping: None,
            friction:        Some(0.6),
            linear_damping:  None,
            mass:            None,
            restitution:     Some(0.35),
        }))?;

        if let Err(err) = set_linear_velocity(prim, velocity) {
            println!("planted body kept its momentum? no: {err:?}");
        }

        Ok(recycled)
    }
}

const fn hidden() -> Xform {
    Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }
}

const fn material(style: Style) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(AlphaMode::Opaque),
        base_color:   Some(Color {
            r: style.color.r,
            g: style.color.g,
            b: style.color.b,
            a: 1.0,
        }),
        double_sided: Some(false),
        emissive:     Some(Color {
            r: style.color.r * style.emissive,
            g: style.color.g * style.emissive,
            b: style.color.b * style.emissive,
            a: 1.0,
        }),
        metallic:     None,
        roughness:    None,
    }
}
