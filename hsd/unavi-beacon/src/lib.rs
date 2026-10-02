use std::{
    f32::consts::TAU,
    str::FromStr,
};

use blake3::Hash;
use wired_guest::{
    color::generate_color,
    math::{
        Color,
        Transform,
        Vec3,
    },
};

use crate::{
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
        peer::authority::is_owner,
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

wired_guest::generate_script!(Script);

const CHANNEL: &str = "unavi:beacon/id";
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

fn build_shell(
    doc: &Document,
    parent: (u64, u64),
    id: Hash,
) -> anyhow::Result<((u64, u64), (u64, u64))> {
    let shape = Cuboid::new(Vec3::splat(SIZE));
    let group = doc.create_prim(wired::scene::document::Layer::Local, Some(parent))?;

    let mut batch = doc
        .local()
        .set(group, Property::Collider(shape.collider()))
        .set(group, Property::RigidBody(RigidBody::dynamic()));

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

/// No space-named prim means the authored template, which does nothing; a
/// real beacon is minted as a document of its own.
struct Script(Option<Beacon>);

struct Beacon {
    doc:          Document,
    color:        Color,
    core:         (u64, u64),
    group:        (u64, u64),
    id:           Hash,
    input:        InputSubscription,
    emit_elapsed: f32,
    published:    bool,
    pulse_step:   u32,
    tick:         u32,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;

        let Some((id, prim)) = doc
            .prims()
            .into_iter()
            .find_map(|p| Hash::from_str(&prim_name(&doc, p)?).ok().map(|id| (id, p)))
        else {
            return Ok(Self(None));
        };

        let existing_group = doc.children(prim).into_iter().next();
        let (group, core) = if let Some(group) = existing_group {
            let core = doc
                .children(group)
                .into_iter()
                .find(|&c| prim_name(&doc, c).as_deref() == Some(CORE_NAME))
                .ok_or_else(|| anyhow::anyhow!("beacon core prim not found"))?;
            (group, core)
        } else {
            build_shell(&doc, prim, id)?
        };

        let input = targeted::listen(&doc, group)?;
        println!("Beacon initialized: space={id}");
        Ok(Self(Some(Beacon {
            doc,
            color: generate_color(id),
            core,
            group,
            id,
            input,
            emit_elapsed: 0.0,
            published: false,
            pulse_step: u32::MAX,
            tick: 0,
        })))
    }

    fn fixed_update(
        &mut self,
        tick: exports::wired::script::lifecycle::Tick,
    ) -> anyhow::Result<()> {
        self.0
            .as_mut()
            .map_or_else(|| Ok(()), |beacon| beacon.fixed_update(tick.dt))
    }
}

impl Beacon {
    fn fixed_update(&mut self, dt: f32) -> anyhow::Result<()> {
        for event in self.input.drain(8) {
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

        if !is_owner(&self.doc)? {
            return Ok(());
        }

        self.pulse();

        self.emit_elapsed += dt;
        if self.emit_elapsed < EMIT_INTERVAL {
            return Ok(());
        }
        self.emit_elapsed = 0.0;

        wired::event::messaging::emit(
            CHANNEL,
            self.id.as_bytes(),
            None,
            Scope::Spatial(Spatial {
                origin: wired::core::ids::PrimRef {
                    document: self.doc.id(),
                    prim:     self.group,
                },
                radius: EVENT_RADIUS,
            }),
        )?;
        Ok(())
    }

    fn pulse(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        let phase = (self.tick % PULSE_TICKS) as f32 / PULSE_TICKS as f32;
        let level = (phase * TAU).sin().mul_add(0.5, 0.5);
        let step = (level * PULSE_LEVELS as f32).round() as u32;
        if step == self.pulse_step {
            return;
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
            .flush()
            .ok();
    }
}
