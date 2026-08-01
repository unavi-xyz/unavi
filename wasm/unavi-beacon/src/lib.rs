use std::{
    f32::consts::TAU,
    str::FromStr,
    time::{
        Duration,
        SystemTime,
    },
};

use blake3::Hash;
use unavi_script_util::color::generate_color;
use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        event::types::{
            EventFilter,
            EventScope,
            SpatialScope,
        },
        input::{
            api::register_input_listener,
            types::{
                InputAction,
                InputListener,
            },
        },
        scene::{
            api::{
                publish_document,
                self_document,
            },
            types::{
                Document,
                Material,
                Prim,
                RigidBody,
                RigidBodyKind,
                Xform,
            },
        },
    },
};

wired_prelude::generate_script!(Script);

const CHANNEL: &str = "unavi::beacon::id";
const EMIT_INTERVAL: Duration = Duration::from_secs(3);

const SIZE: f32 = 0.095;
const EVENT_RADIUS: f32 = SIZE * 3.0;

// Larger corners leave only a thin gap between them; the recessed core cube
// shows through that negative space as a 2D cross on each face.
const CORNER: f32 = SIZE * 0.44;
const CORNER_OFFSET: f32 = SIZE * 0.5 - CORNER * 0.5;
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

const IDENTITY_QUAT: Quat = Quat {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};

fn set_translation(prim: &Prim, translation: Vec3) {
    prim.set_xform(Some(Xform {
        translation,
        rotation: IDENTITY_QUAT,
        scale: Vec3::splat(1.0),
    }))
    .ok();
}

const fn material(color: Color, emissive_scale: f32) -> Material {
    Material {
        alpha_cutoff:               None,
        alpha_mode:                 None,
        base_color:                 Some(color),
        base_color_texture:         None,
        double_sided:               None,
        emissive:                   Some(Color {
            r: color.r * emissive_scale,
            g: color.g * emissive_scale,
            b: color.b * emissive_scale,
            a: color.a,
        }),
        emissive_texture:           None,
        metallic:                   Some(0.3),
        metallic_roughness_texture: None,
        normal_texture:             None,
        occlusion_texture:          None,
        roughness:                  Some(0.7),
    }
}

/// A cube shell with its 8 corners as small dark cubes, sized so only a thin
/// gap separates neighbors. A single recessed core cube sits behind that
/// gap, its color showing through as a 2D inset cross on every face. Built
/// once per document; a peer that already holds a published beacon reuses
/// the synced shell rather than authoring a copy.
fn build_shell(doc: &Document, parent: &Prim, id: Hash) -> Prim {
    let shape = Cuboid::new(Vec3::splat(SIZE));
    let group = doc.create_prim().expect("create_prim");
    group.set_collider(Some(&shape.collider())).ok();
    group
        .set_rigid_body(Some(RigidBody {
            kind:            RigidBodyKind::Dynamic,
            angular_damping: None,
            friction:        None,
            linear_damping:  None,
            mass:            None,
            restitution:     None,
        }))
        .ok();

    let shell_mat = material(SHELL_COLOR, 0.0);
    for x in [-1.0_f32, 1.0] {
        for y in [-1.0_f32, 1.0] {
            for z in [-1.0_f32, 1.0] {
                let corner = Cuboid::new(Vec3::splat(CORNER)).mesh();
                corner.set_material(Some(&shell_mat)).ok();
                set_translation(&corner, Vec3::new(x, y, z) * CORNER_OFFSET);
                group.add_child(&corner).ok();
            }
        }
    }

    let core = Cuboid::new(Vec3::splat(CORE_SIZE)).mesh();
    core.set_material(Some(&material(generate_color(id), PULSE_MIN_EMISSIVE)))
        .ok();
    core.set_name(Some(CORE_NAME)).ok();
    group.add_child(&core).ok();

    parent.add_child(&group).ok();
    group
}

struct Script {
    color:      Color,
    core:       Prim,
    group:      Prim,
    id:         Hash,
    input:      InputListener,
    emit_time:  SystemTime,
    published:  bool,
    pulse_step: u32,
    tick:       u32,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = self_document()?;

        let Some((id, prim)) = doc.prims().into_iter().find_map(|p| {
            Hash::from_str(&p.name().unwrap_or_default())
                .ok()
                .map(|id| (id, p))
        }) else {
            panic!("invalid beacon: id prim not found")
        };

        let group = prim
            .children()
            .into_iter()
            .next()
            .unwrap_or_else(|| build_shell(&doc, &prim, id));
        let core = group
            .children()
            .into_iter()
            .find(|c| c.name().as_deref() == Some(CORE_NAME))
            .expect("beacon core prim");

        let input = register_input_listener(&group)?;
        println!("Beacon initialized: space={id}");
        Ok(Self {
            color: generate_color(id),
            core,
            group,
            id,
            input,
            emit_time: SystemTime::now(),
            published: false,
            pulse_step: u32::MAX,
            tick: 0,
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.input.poll() {
            if !self.published && matches!(event.action, InputAction::GrabDown) {
                let doc = self_document()?;
                match publish_document(&doc.id()) {
                    Ok(()) => {
                        self.published = true;
                        println!("Beacon published: space={}", self.id);
                    }
                    Err(err) => eprintln!("Beacon publish failed: {err:?}"),
                }
            }
        }

        if !wired::peer::api::is_self_owner()? {
            return Ok(());
        }

        self.pulse();

        if self.emit_time.elapsed().expect("elapsed") < EMIT_INTERVAL {
            return Ok(());
        }
        self.emit_time = SystemTime::now();

        wired::event::api::emit(
            CHANNEL,
            self.id.as_bytes(),
            EventFilter {
                documents: None,
                scope:     EventScope::Spatial(SpatialScope {
                    prim:   self.group.clone(),
                    radius: EVENT_RADIUS,
                }),
            },
        )?;
        Ok(())
    }
}

impl Script {
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
        self.core
            .set_material(Some(&material(self.color, emissive)))
            .ok();
    }
}
