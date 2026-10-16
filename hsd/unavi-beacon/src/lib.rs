//! A beacon: a small pulsing marker planted by halo's `Nav` that announces
//! the space it was copied for over [`CHANNEL`] so a gate nearby can open to
//! it. The authored template prim does nothing; a real beacon is a document
//! of its own, minted by copying the template and naming a prim for its
//! space.

use std::{
    f32::consts::TAU,
    str::FromStr,
};

use blake3::Hash;
use wired_guest::{
    beacon::{
        PLANTED_KEY,
        age_secs,
        now_ms,
    },
    color::generate_color,
    math::{
        Color,
        Transform,
        Vec3,
    },
};

use crate::{
    float::Float,
    unavi::shapes::api::Cuboid,
    wired::{
        event::messaging::{
            Scope,
            Spatial,
        },
        input::{
            targeted,
            types::{
                Action,
                Button,
                InputSubscription,
            },
        },
        peer::authority::is_author,
        physics::simulation::set_velocity,
        scene::{
            document::{
                Document,
                script_document,
            },
            properties::{
                Material,
                Property,
                PropertyKey,
                RigidBody,
            },
        },
    },
};

mod float;

wired_guest::generate_script!(Script);

const EMIT_INTERVAL: f32 = 3.0;

const SIZE: f32 = 0.095;
const EVENT_RADIUS: f32 = SIZE * 3.0;

// Larger corners leave only a thin gap between them; the recessed core cube
// shows through that negative space as a 2D cross on each face.
const CORNER: f32 = SIZE * 0.44;
const CORNER_OFFSET: f32 = (SIZE - CORNER) * 0.5;
const CORE_SIZE: f32 = SIZE * 0.84;
const CORE_NAME: &str = "core";

const SHELL_COLOR: Color = Color {
    r: 0.05,
    g: 0.05,
    b: 0.06,
    a: 1.0,
};

const PULSE_TICKS: u32 = 90;
const PULSE_LEVELS: u32 = 12;
const PULSE_MIN_EMISSIVE: f32 = 0.25;
const PULSE_MAX_EMISSIVE: f32 = 0.95;

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

fn prim_name(doc: &Document, prim: (u64, u64)) -> Option<String> {
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
/// the grab only takes hold of a dynamic body. `hover` holds it in the air
/// and [`Beacon::release`] gives gravity back.
fn make_body(doc: &Document, body: (u64, u64), floating: bool) -> anyhow::Result<()> {
    let mut batch = doc.local().set(
        body,
        Property::Collider(Cuboid::new(Vec3::splat(SIZE)).collider()),
    );
    batch = batch.set(body, Property::RigidBody(RigidBody::dynamic()));
    if floating {
        batch = batch.set(body, Property::GravityScale(0.0));
    }
    batch.flush()?;
    Ok(())
}

/// Builds the visible shell under the body. Its prims are this peer's own, so
/// they carry no physics and only follow the body.
fn build_shell(
    doc: &Document,
    parent: (u64, u64),
    id: Hash,
) -> anyhow::Result<((u64, u64), (u64, u64))> {
    let group = doc.create_prim(wired::scene::document::Layer::Local, Some(parent))?;

    let mut batch = doc.local();

    let shell_mat = material(SHELL_COLOR, 0.0);
    for x in [-1.0_f32, 1.0] {
        for y in [-1.0_f32, 1.0] {
            for z in [-1.0_f32, 1.0] {
                let corner_shape = Cuboid::new(Vec3::splat(CORNER));
                let corner = corner_shape.mesh()?;
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

    let core_shape = Cuboid::new(Vec3::splat(CORE_SIZE));
    let core = core_shape.mesh()?;
    batch = batch
        .set(
            core,
            Property::Material(material(generate_color(id), PULSE_MIN_EMISSIVE)),
        )
        .set(core, Property::Name(CORE_NAME.to_owned()))
        .set(core, Property::Parent(Some(group)));

    batch.flush()?;

    Ok((group, core))
}

/// Ticks between looks for the space-named prim while none is found.
const FIND_INTERVAL: u32 = 30;

/// No space-named prim means the authored template, which does nothing, or a
/// minted beacon whose committed prim has not synced to this peer yet, which
/// is looked for again every [`FIND_INTERVAL`] ticks.
struct Script {
    beacon: Option<Beacon>,
    waited: u32,
}

struct Beacon {
    doc:          Document,
    color:        Color,
    core:         (u64, u64),
    /// The space-named prim, which carries the collider and rigid body.
    body:         (u64, u64),
    id:           Hash,
    input:        InputSubscription,
    float:        Float,
    /// The body's pose when it was let go, which the hover moves from.
    rest:         Option<Transform>,
    emit_elapsed: f32,
    published:    bool,
    pulse_step:   u32,
    tick:         u32,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            beacon: Beacon::find()?,
            waited: 0,
        })
    }

    fn fixed_update(&mut self, tick: Tick) -> anyhow::Result<()> {
        if self.beacon.is_none() {
            self.waited += 1;
            if self.waited < FIND_INTERVAL {
                return Ok(());
            }
            self.waited = 0;
            self.beacon = Beacon::find()?;
        }
        self.beacon
            .as_mut()
            .map_or_else(|| Ok(()), |beacon| beacon.fixed_update(tick.dt))
    }
}

impl Beacon {
    /// The beacon this script's document is, once its space-named prim is
    /// present.
    fn find() -> anyhow::Result<Option<Self>> {
        let doc = script_document()?;

        let Some((id, prim)) = doc
            .prims()
            .into_iter()
            .find_map(|p| Hash::from_str(&prim_name(&doc, p)?).ok().map(|id| (id, p)))
        else {
            return Ok(None);
        };

        let rest = match doc.get(prim, &PropertyKey::Transform) {
            Some(Property::Transform(transform)) => Some(transform),
            _ => None,
        };
        // The age comes from a stamp every peer reads, so a late joiner
        // finds the hover where it is, and a beacon that has fallen does not
        // float again. A beacon with no stamp has nothing to hover from.
        let float = match doc.get(prim, &PropertyKey::Custom(PLANTED_KEY.to_owned())) {
            Some(Property::Custom((_, stamp))) => {
                age_secs(&stamp, now_ms()).map_or_else(Float::over, Float::at_age)
            }
            _ => Float::over(),
        };
        make_body(&doc, prim, float.is_floating())?;
        let existing_group = doc.children(prim).into_iter().next();
        let (_, core) = if let Some(group) = existing_group {
            let core = doc
                .children(group)
                .into_iter()
                .find(|&c| prim_name(&doc, c).as_deref() == Some(CORE_NAME))
                .ok_or_else(|| anyhow::anyhow!("beacon core prim not found"))?;
            (group, core)
        } else {
            build_shell(&doc, prim, id)?
        };

        let input = targeted::listen(&doc, prim)?;
        println!("Beacon initialized: space={id}");
        Ok(Some(Self {
            doc,
            color: generate_color(id),
            core,
            body: prim,
            id,
            input,
            float,
            rest,
            emit_elapsed: 0.0,
            published: false,
            pulse_step: u32::MAX,
            tick: 0,
        }))
    }

    /// Holds the body on its hover path. Every peer does the same, since
    /// every peer's simulation steps the body; the holder's motion is the
    /// one that wins, and it is this same path.
    fn hover(&self) {
        let Some(rest) = self.rest else {
            return;
        };
        let Ok(now) = self.doc.world_transform(self.body) else {
            return;
        };
        let drive = self.float.drive(rest, now);
        // Fails until the body is in the simulation, which is the next tick.
        let _ = set_velocity(
            &self.doc,
            self.body,
            Some(drive.linear),
            Some(drive.angular),
        );
    }

    /// Ends the hover: gravity comes back and the body falls from where it
    /// is.
    fn release(&self) -> anyhow::Result<()> {
        self.doc
            .local()
            .set(self.body, Property::GravityScale(1.0))
            .flush()?;
        Ok(())
    }

    fn fixed_update(&mut self, dt: f32) -> anyhow::Result<()> {
        let mut grabbed = false;
        for event in self.input.drain(8) {
            // Only the grip takes hold of a body; the trigger is the tools'.
            grabbed |= matches!(event.action, Action::Pressed(Button::Grip));
            if !self.published
                && matches!(
                    event.action,
                    Action::Pressed(Button::Trigger | Button::Grip)
                )
            {
                // The copied template is already durable in its own
                // namespace, so pressing the beacon only marks it published.
                self.published = true;
                println!("Beacon published: space={}", self.id);
            }
        }

        if self.float.step(dt, grabbed) {
            self.release()?;
        } else if self.float.is_floating() {
            self.hover();
        }

        if !is_author(&self.doc)? {
            return Ok(());
        }

        self.pulse()?;

        self.emit_elapsed += dt;
        if self.emit_elapsed < EMIT_INTERVAL {
            return Ok(());
        }
        self.emit_elapsed = 0.0;

        wired::event::messaging::emit(
            wired_guest::beacon::CHANNEL,
            self.id.as_bytes(),
            None,
            Scope::Spatial(Spatial {
                origin: wired::core::ids::PrimRef {
                    document: self.doc.id(),
                    prim:     self.body,
                },
                radius: EVENT_RADIUS,
            }),
        )?;
        Ok(())
    }

    fn pulse(&mut self) -> anyhow::Result<()> {
        self.tick = self.tick.wrapping_add(1);
        let phase = (self.tick % PULSE_TICKS) as f32 / PULSE_TICKS as f32;
        let level = (phase * TAU).sin().mul_add(0.5, 0.5);
        let step = (level * PULSE_LEVELS as f32).round() as u32;
        if step == self.pulse_step {
            return Ok(());
        }
        self.pulse_step = step;

        let emissive = (step as f32 / PULSE_LEVELS as f32)
            .mul_add(PULSE_MAX_EMISSIVE - PULSE_MIN_EMISSIVE, PULSE_MIN_EMISSIVE);
        self.doc
            .local()
            .set(
                self.core,
                Property::Material(material(self.color, emissive)),
            )
            .flush()?;
        Ok(())
    }
}
