//! The body of whatever is in hand, carried in front of the viewer.
//!
//! On desktop there is no tracked hand to put a held tool in, so this stands
//! in for it: a glowing core with orbiters, wearing the held tool's own
//! colour, shown only while something is held.
//!
//! It is also load-bearing rather than decorative. A tool fires from
//! [`ARTIFACT_OFFSET`] — the physgun's muzzle *is* this body — so without it a
//! beam starts in empty air and nothing says a tool is running at all.
//!
//! It retires when a tracked hand can hold the tool itself.

use std::f32::consts::TAU;

use wired_guest::{
    math::{
        Color,
        Quat,
        Transform,
        Vec3,
    },
    xform::{
        ARTIFACT_OFFSET,
        hidden,
        yaw,
    },
};

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::{
            Document,
            Layer,
            script_document,
        },
        properties::{
            Material,
            Property,
        },
    },
};

const CORE_SIZE: f32 = 0.045;
const ORBITER_SIZE: f32 = 0.018;
const ORBITERS: usize = 3;
const ORBIT_RADIUS: f32 = 0.06;
const SPIN_IDLE: f32 = 0.8;
const SPIN_ACTIVE: f32 = 3.0;
const TILT: f32 = 0.35;
/// How fast it grows in and out of the hand.
const SPEED: f32 = 5.0;

pub struct Artifact {
    doc:      Document,
    root:     (u64, u64),
    core:     (u64, u64),
    orbiters: Vec<(u64, u64)>,
    spin:     f32,
    /// How far out it is, 0 to 1, so appearing and going away are the same
    /// motion run in opposite directions.
    out:      f32,
}

impl Artifact {
    pub fn new() -> anyhow::Result<Self> {
        let doc = script_document()?;
        let root = doc.create_prim(Layer::Local, None)?;
        let core = Cuboid::new(Vec3::splat(CORE_SIZE)).mesh()?;

        let mut batch = doc
            .local()
            .set(root, Property::Transform(hidden()))
            .set(core, Property::Parent(Some(root)))
            .set(core, Property::Transform(hidden()));

        let mut orbiters = Vec::with_capacity(ORBITERS);
        for _ in 0..ORBITERS {
            let orbiter = Cuboid::new(Vec3::splat(ORBITER_SIZE)).mesh()?;
            batch = batch
                .set(orbiter, Property::Parent(Some(root)))
                .set(orbiter, Property::Transform(hidden()));
            orbiters.push(orbiter);
        }
        batch.flush()?;

        Ok(Self {
            doc,
            root,
            core,
            orbiters,
            spin: 0.0,
            out: 0.0,
        })
    }

    /// Dresses it in the held tool's colour. Materials are written only when
    /// the tool changes, never per frame.
    pub fn wear(&self, color: Color) {
        let mut batch = self
            .doc
            .local()
            .set(self.core, Property::Material(lit(color, 0.6)));
        for &orbiter in &self.orbiters {
            batch = batch.set(orbiter, Property::Material(lit(color, 0.75)));
        }
        if let Err(err) = batch.flush() {
            eprintln!("halo: wear artifact: {err:?}");
        }
    }

    /// Rides the viewer, spinning faster the further out it is.
    pub fn update(&mut self, eye: &Transform, held: bool, delta: f32) -> anyhow::Result<()> {
        let step = delta * SPEED;
        self.out = if held {
            (self.out + step).min(1.0)
        } else {
            (self.out - step).max(0.0)
        };

        self.doc
            .local()
            .set(
                self.root,
                Property::Transform(Transform {
                    translation: eye.translation + eye.rotation * ARTIFACT_OFFSET,
                    rotation:    eye.rotation,
                    scale:       Vec3::splat(self.out),
                }),
            )
            .flush()?;
        if self.out <= 0.0 {
            return Ok(());
        }

        self.spin = delta.mul_add(
            self.out.mul_add(SPIN_ACTIVE - SPIN_IDLE, SPIN_IDLE),
            self.spin,
        );
        let mut batch = self.doc.local().set(
            self.core,
            Property::Transform(Transform {
                translation: Vec3::ZERO,
                rotation:    yaw(self.spin),
                scale:       Vec3::ONE,
            }),
        );
        for (index, &orbiter) in self.orbiters.iter().enumerate() {
            let phase = self.spin + index as f32 * TAU / ORBITERS as f32;
            batch = batch.set(
                orbiter,
                Property::Transform(Transform {
                    translation: Vec3::new(
                        ORBIT_RADIUS * phase.cos(),
                        ORBIT_RADIUS * TILT * (phase * 2.0).sin(),
                        ORBIT_RADIUS * phase.sin(),
                    ),
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::ONE,
                }),
            );
        }
        batch.flush()?;
        Ok(())
    }
}

const fn lit(color: Color, glow: f32) -> Material {
    Material {
        base_color:   Some(color),
        emissive:     Some(Color {
            r: color.r * glow,
            g: color.g * glow,
            b: color.b * glow,
            a: 1.0,
        }),
        metallic:     Some(0.0),
        roughness:    Some(0.5),
        alpha_mode:   None,
        alpha_cutoff: None,
        double_sided: None,
    }
}
