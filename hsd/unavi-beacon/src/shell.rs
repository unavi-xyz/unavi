//! What a beacon looks like: eight dark corners around a core in its space's
//! colour, whose gaps draw a cross on every face.

use std::f32::consts::TAU;

use blake3::Hash;
use wired_guest::{
    beacon::BEACON_SIZE,
    color::generate_color,
    math::{
        Color,
        Transform,
        Vec3,
    },
};

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::{
            Material,
            Property,
            PropertyKey,
            RigidBody,
        },
    },
};

// Larger corners leave only a thin gap between them; the recessed core cube
// shows through that negative space as a 2D cross on each face.
const CORNER: f32 = BEACON_SIZE * 0.44;
const CORNER_OFFSET: f32 = (BEACON_SIZE - CORNER) * 0.5;
const CORE_SIZE: f32 = BEACON_SIZE * 0.84;
const CORE_NAME: &str = "core";
/// How far an armed beacon's corners spread, as a multiple of their resting
/// offset, opening the cross on every face.
const ARMED_SPREAD: f32 = 1.35;

const SHELL_COLOR: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.06,
    a: 1.0,
};

/// Seconds per pulse while resting.
const PULSE_SECS: f32 = 3.0;
/// A beacon held near a socket that would take it pulses this much faster.
const EAGER_SECS: f32 = 0.8;
const PULSE_LEVELS: u32 = 32;
const PULSE_MIN_EMISSIVE: f32 = 0.25;
const PULSE_MAX_EMISSIVE: f32 = 0.95;
const SEATED_EMISSIVE: f32 = 1.2;

const fn material(color: Color, emissive_scale: f32) -> Material {
    Material {
        base_color:   Some(color),
        emissive:     Some(Color {
            r: color.r * emissive_scale,
            g: color.g * emissive_scale,
            b: color.b * emissive_scale,
            a: color.a,
        }),
        metallic:     Some(0.3),
        roughness:    Some(0.7),
        alpha_mode:   None,
        alpha_cutoff: None,
        double_sided: None,
    }
}

pub fn prim_name(doc: &Document, prim: (u64, u64)) -> Option<String> {
    match doc.get(prim, &PropertyKey::Name) {
        Some(Property::Name(name)) => Some(name),
        _ => None,
    }
}

/// Makes the space-named prim the beacon's body. Every peer's copy of the
/// script does the same to the same committed prim, so the holder's pose
/// stream drives one body everywhere.
///
/// A hovering body is dynamic with no gravity rather than kinematic, because
/// the grab only takes hold of a dynamic body.
pub fn make_body(doc: &Document, body: (u64, u64), floating: bool) -> anyhow::Result<()> {
    let mut batch = doc
        .local()
        .set(
            body,
            Property::Collider(Cuboid::new(Vec3::splat(BEACON_SIZE)).collider()),
        )
        .set(body, Property::RigidBody(RigidBody::dynamic()));
    if floating {
        batch = batch.set(body, Property::GravityScale(0.0));
    }
    batch.flush()?;
    Ok(())
}

/// What the core is saying.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Resting,
    /// Held near a socket that would take it.
    Eager,
    /// In a socket.
    Seated,
}

pub struct Shell {
    core:    (u64, u64),
    /// Each corner, with the signs of its offset on each axis.
    corners: Vec<((u64, u64), Vec3)>,
    color:   Color,
    /// How far through its pulse the core is, from 0 to 1.
    phase:   f32,
    shown:   Option<u32>,
    spread:  Option<bool>,
}

impl Shell {
    /// The shell under `body`, built once per peer. Its prims are this peer's
    /// own, so they carry no physics and only follow the body.
    pub fn under(doc: &Document, body: (u64, u64), id: Hash) -> anyhow::Result<Self> {
        let color = generate_color(id);
        let group = match doc.children(body).into_iter().next() {
            Some(group) => group,
            None => build(doc, body, color)?,
        };
        let mut core = None;
        let mut corners = Vec::with_capacity(8);
        for child in doc.children(group) {
            if prim_name(doc, child).as_deref() == Some(CORE_NAME) {
                core = Some(child);
            } else if let Some(Property::Transform(at)) = doc.get(child, &PropertyKey::Transform) {
                corners.push((child, at.translation.signum()));
            }
        }
        let core = core.ok_or_else(|| anyhow::anyhow!("beacon core prim not found"))?;
        Ok(Self {
            core,
            corners,
            color,
            phase: 0.0,
            shown: None,
            spread: None,
        })
    }

    /// Spreads the corners while `armed`, and closes them again after.
    pub fn set_armed(&mut self, doc: &Document, armed: bool) -> anyhow::Result<()> {
        if self.spread == Some(armed) {
            return Ok(());
        }
        self.spread = Some(armed);
        let offset = if armed {
            CORNER_OFFSET * ARMED_SPREAD
        } else {
            CORNER_OFFSET
        };
        let mut batch = doc.local();
        for &(corner, signs) in &self.corners {
            batch = batch.set(
                corner,
                Property::Transform(Transform::from_translation(signs * offset)),
            );
        }
        batch.flush()?;
        Ok(())
    }

    /// Steps the core's light by `dt` seconds.
    pub fn update(&mut self, doc: &Document, mood: Mood, dt: f32) -> anyhow::Result<()> {
        let step = match mood {
            Mood::Seated => PULSE_LEVELS + 1,
            Mood::Resting | Mood::Eager => {
                let period = if mood == Mood::Eager {
                    EAGER_SECS
                } else {
                    PULSE_SECS
                };
                self.phase = (self.phase + dt / period).fract();
                let level = (self.phase * TAU).sin().mul_add(0.5, 0.5);
                (level * PULSE_LEVELS as f32).round() as u32
            }
        };
        if self.shown == Some(step) {
            return Ok(());
        }
        self.shown = Some(step);
        let emissive = if step > PULSE_LEVELS {
            SEATED_EMISSIVE
        } else {
            (step as f32 / PULSE_LEVELS as f32)
                .mul_add(PULSE_MAX_EMISSIVE - PULSE_MIN_EMISSIVE, PULSE_MIN_EMISSIVE)
        };
        doc.local()
            .set(
                self.core,
                Property::Material(material(self.color, emissive)),
            )
            .flush()?;
        Ok(())
    }
}

/// Builds the corners and core under a new group beneath `parent`, and
/// answers the group.
fn build(doc: &Document, parent: (u64, u64), color: Color) -> anyhow::Result<(u64, u64)> {
    let group = doc.create_prim(Layer::Local, Some(parent))?;
    let mut batch = doc.local();

    let shell_mat = material(SHELL_COLOR, 0.0);
    for x in [-1.0_f32, 1.0] {
        for y in [-1.0_f32, 1.0] {
            for z in [-1.0_f32, 1.0] {
                let corner = Cuboid::new(Vec3::splat(CORNER)).mesh()?;
                batch = batch
                    .set(corner, Property::Material(shell_mat))
                    .set(
                        corner,
                        Property::Transform(Transform::from_translation(
                            Vec3::new(x, y, z) * CORNER_OFFSET,
                        )),
                    )
                    .set(corner, Property::Parent(Some(group)));
            }
        }
    }

    let core = Cuboid::new(Vec3::splat(CORE_SIZE)).mesh()?;
    batch = batch
        .set(
            core,
            Property::Material(material(color, PULSE_MIN_EMISSIVE)),
        )
        .set(core, Property::Name(CORE_NAME.to_owned()))
        .set(core, Property::Parent(Some(group)));
    batch.flush()?;
    Ok(group)
}
