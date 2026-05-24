use std::{
    f32::consts::GOLDEN_RATIO,
    time::{Duration, SystemTime},
};

use blake3::Hash;
use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        event::types::{EventFilter, EventReceptor, EventScope, SpatialScope},
        scene::{
            api::self_document,
            types::{Material, Prim, RigidBody, RigidBodyKind, Xform},
        },
    },
};

wired_prelude::generate_script!(Script);

const CHANNEL: &str = "unavi::beacon::id";

const PORTAL_WIDTH: f32 = GOLDEN_RATIO;
const PORTAL_HEIGHT: f32 = PORTAL_WIDTH * GOLDEN_RATIO;

const BEAM_THICKNESS: f32 = 1.0 / (4.0 * GOLDEN_RATIO);

const PEDESTAL_HEIGHT: f32 = PORTAL_WIDTH / 2.0;
const PEDESTAL_THICKNESS: f32 = BEAM_THICKNESS * GOLDEN_RATIO;
const EVENT_RADIUS: f32 = PEDESTAL_THICKNESS * 2.0;

const TARGET_DECAY: Duration = Duration::from_secs(10);

const fn static_body() -> RigidBody {
    RigidBody {
        kind: RigidBodyKind::Static,
        angular_damping: None,
        friction: None,
        linear_damping: None,
        mass: None,
        restitution: None,
    }
}

fn set_translation(prim: &Prim, translation: Vec3) {
    prim.set_xform(Some(Xform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }));
}

const fn gate_material() -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode: None,
        base_color: Some(Color {
            r: 0.7,
            g: 0.72,
            b: 0.78,
            a: 1.0,
        }),
        base_color_texture: None,
        double_sided: None,
        emissive: None,
        emissive_texture: None,
        metallic: Some(0.6),
        metallic_roughness_texture: None,
        normal_texture: None,
        occlusion_texture: None,
        roughness: Some(0.4),
    }
}

struct Script {
    receptor: EventReceptor,
    target: Option<(Hash, SystemTime)>,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();
        let root = doc.roots().into_iter().next().expect("root");

        let material = gate_material();

        let pole = Cuboid::new(Vec3::new(BEAM_THICKNESS, PORTAL_HEIGHT, BEAM_THICKNESS));

        let pole_l = pole.mesh();
        root.add_child(&pole_l);
        pole_l.set_collider(Some(&pole.collider()));
        pole_l.set_rigid_body(Some(static_body()));
        pole_l.set_material(Some(&material));
        set_translation(
            &pole_l,
            Vec3::new(
                -PORTAL_WIDTH / 2.0 - BEAM_THICKNESS / 2.0,
                PORTAL_HEIGHT / 2.0,
                0.0,
            ),
        );

        let pole_r = pole.mesh();
        root.add_child(&pole_r);
        pole_r.set_collider(Some(&pole.collider()));
        pole_r.set_rigid_body(Some(static_body()));
        pole_r.set_material(Some(&material));
        set_translation(
            &pole_r,
            Vec3::new(
                PORTAL_WIDTH / 2.0 + BEAM_THICKNESS / 2.0,
                PORTAL_HEIGHT / 2.0,
                0.0,
            ),
        );

        let beam = Cuboid::new(Vec3::new(
            BEAM_THICKNESS.mul_add(2.0, PORTAL_WIDTH),
            BEAM_THICKNESS,
            BEAM_THICKNESS,
        ));

        let beam_top = beam.mesh();
        root.add_child(&beam_top);
        beam_top.set_collider(Some(&beam.collider()));
        beam_top.set_rigid_body(Some(static_body()));
        beam_top.set_material(Some(&material));
        set_translation(
            &beam_top,
            Vec3::new(0.0, PORTAL_HEIGHT + BEAM_THICKNESS / 2.0, 0.0),
        );

        let pedestal_shape = Cuboid::new(Vec3::new(
            PEDESTAL_THICKNESS,
            PEDESTAL_HEIGHT,
            PEDESTAL_THICKNESS,
        ));

        let pedestal = pedestal_shape.mesh();
        root.add_child(&pedestal);
        pedestal.set_collider(Some(&pedestal_shape.collider()));
        pedestal.set_rigid_body(Some(static_body()));
        pedestal.set_material(Some(&material));
        set_translation(
            &pedestal,
            Vec3::new(-PORTAL_WIDTH, PEDESTAL_HEIGHT / 2.0, 0.0),
        );

        let receptor_prim = doc.create_prim();
        pedestal.add_child(&receptor_prim);
        set_translation(&receptor_prim, Vec3::new(0.0, PEDESTAL_HEIGHT, 0.0));

        let receptor = wired::event::api::listen(
            &[CHANNEL.to_string()],
            EventFilter {
                documents: None,
                scope: EventScope::Spatial(SpatialScope {
                    prim: receptor_prim,
                    radius: EVENT_RADIUS,
                }),
            },
        );

        println!("Gate ready");

        Self {
            receptor,
            target: None,
        }
    }

    fn tick(&mut self) {
        if let Some((_, t)) = &self.target
            && t.elapsed().expect("elapsed") >= TARGET_DECAY
        {
            self.target = None;
        }

        while let Some(event) = self.receptor.poll() {
            let Ok(id) = Hash::from_slice(&event.payload) else {
                continue;
            };
            if self.target.as_ref().is_some_and(|(x, _)| *x != id) {
                continue;
            }
            println!("Loading beacon: {id}");
            self.target = Some((id, SystemTime::now()));
        }
    }
}
