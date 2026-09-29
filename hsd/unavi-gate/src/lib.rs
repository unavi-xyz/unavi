use std::f32::consts::GOLDEN_RATIO;

use anyhow::Context;
use unavi_portal_protocol::{
    INTENT_CHANNEL,
    LinkIntent,
};
use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        event::types::{
            EventFilter,
            EventReceptor,
            EventScope,
            SpatialScope,
        },
        scene::{
            api::self_document,
            types::{
                Document,
                Material,
                Portal,
                Prim,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

wired_prelude::generate_script!(Script);

const CHANNEL: &str = "unavi::beacon::id";
const PORTAL_PRIM_NAME: &str = "portal";
const RECEPTOR_PRIM_NAME: &str = "receptor";

const PORTAL_WIDTH: f32 = GOLDEN_RATIO;
const PORTAL_HEIGHT: f32 = PORTAL_WIDTH * GOLDEN_RATIO;

const BEAM_THICKNESS: f32 = 1.0 / (4.0 * GOLDEN_RATIO);

const PEDESTAL_HEIGHT: f32 = PORTAL_WIDTH / 2.0;
const PEDESTAL_THICKNESS: f32 = BEAM_THICKNESS * GOLDEN_RATIO;
const EVENT_RADIUS: f32 = PEDESTAL_THICKNESS;

struct Script {
    portal_prim: Prim,
    beacon_rx:   EventReceptor,
    intent_rx:   EventReceptor,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = self_document()?;
        let root = doc.roots().into_iter().next().expect("root");

        // Authored prims have ids baked at build time, identical on every
        // peer; a prim minted at runtime gets a per-peer id.
        let portal_prim = named_prim(&doc, PORTAL_PRIM_NAME)?;
        let receptor_prim = named_prim(&doc, RECEPTOR_PRIM_NAME)?;
        portal_prim.set_portal(Some(&Portal {
            destination: None,
            size_x:      PORTAL_WIDTH,
            size_y:      PORTAL_HEIGHT,
        }))?;

        let material = gate_material();
        spawn_frame(&root, material);

        let pedestal_shape = Cuboid::new(Vec3::new(
            PEDESTAL_THICKNESS,
            PEDESTAL_HEIGHT,
            PEDESTAL_THICKNESS,
        ));

        let pedestal = pedestal_shape.mesh();
        root.add_child(&pedestal)?;
        pedestal.set_collider(Some(pedestal_shape.collider()))?;
        pedestal.set_rigid_body(Some(static_body()))?;
        pedestal.set_material(Some(material))?;
        set_translation(
            &pedestal,
            Vec3::new(-PORTAL_WIDTH, PEDESTAL_HEIGHT / 2.0, 0.0),
        );

        let beacon_rx = wired::event::api::listen(
            &[CHANNEL.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Spatial(SpatialScope {
                    prim:   receptor_prim,
                    radius: EVENT_RADIUS,
                }),
            },
        )?;

        let intent_rx = wired::event::api::listen(
            &[INTENT_CHANNEL.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        )?;

        println!("Gate ready");

        Ok(Self {
            portal_prim,
            beacon_rx,
            intent_rx,
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.beacon_rx.poll() {
            let payload = event.payload();
            let Ok(target) = <[u8; 32]>::try_from(payload.as_slice()) else {
                continue;
            };
            let current = self.portal_prim.portal().and_then(|p| p.destination);
            if current.is_some_and(|d| d.space == target) {
                continue;
            }
            wired::portal::api::open(self.portal_prim.clone(), target.as_ref())?;
        }

        while let Some(event) = self.intent_rx.poll() {
            // A gate already leading somewhere leaves the intent for another.
            let idle = self
                .portal_prim
                .portal()
                .is_none_or(|p| p.destination.is_none());
            if !idle {
                continue;
            }
            let Ok(intent) = postcard::from_bytes::<LinkIntent>(&event.payload()) else {
                continue;
            };
            if !event.consume() {
                continue;
            }
            wired::portal::api::pair(
                self.portal_prim.clone(),
                intent.source_space.as_ref(),
                intent.link.as_ref(),
            )?;
        }
        Ok(())
    }
}

fn named_prim(doc: &Document, name: &str) -> anyhow::Result<Prim> {
    doc.prims()
        .into_iter()
        .find(|p| p.name().is_some_and(|n| n == name))
        .with_context(|| format!("{name} prim not found"))
}

fn spawn_frame(root: &Prim, material: Material) {
    let pole = Cuboid::new(Vec3::new(BEAM_THICKNESS, PORTAL_HEIGHT, BEAM_THICKNESS));

    let pole_l = pole.mesh();
    root.add_child(&pole_l).ok();
    pole_l.set_collider(Some(pole.collider())).ok();
    pole_l.set_rigid_body(Some(static_body())).ok();
    pole_l.set_material(Some(material)).ok();
    set_translation(
        &pole_l,
        Vec3::new(
            -PORTAL_WIDTH / 2.0 - BEAM_THICKNESS / 2.0,
            PORTAL_HEIGHT / 2.0,
            0.0,
        ),
    );

    let pole_r = pole.mesh();
    root.add_child(&pole_r).ok();
    pole_r.set_collider(Some(pole.collider())).ok();
    pole_r.set_rigid_body(Some(static_body())).ok();
    pole_r.set_material(Some(material)).ok();
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
    root.add_child(&beam_top).ok();
    beam_top.set_collider(Some(beam.collider())).ok();
    beam_top.set_rigid_body(Some(static_body())).ok();
    beam_top.set_material(Some(material)).ok();
    set_translation(
        &beam_top,
        Vec3::new(0.0, PORTAL_HEIGHT + BEAM_THICKNESS / 2.0, 0.0),
    );
}

const fn static_body() -> RigidBody {
    RigidBody {
        kind:            RigidBodyKind::Static,
        angular_damping: None,
        friction:        None,
        linear_damping:  None,
        mass:            None,
        restitution:     None,
    }
}

fn set_translation(prim: &Prim, translation: Vec3) {
    prim.set_xform(Some(Transform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }))
    .ok();
}

const fn gate_material() -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(Color {
            r: 0.7,
            g: 0.72,
            b: 0.78,
            a: 1.0,
        }),
        double_sided: None,
        emissive:     None,
        metallic:     Some(0.6),
        roughness:    Some(0.4),
    }
}
