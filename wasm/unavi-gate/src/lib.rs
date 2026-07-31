use std::f32::consts::GOLDEN_RATIO;

use anyhow::Context;
use unavi_portal_protocol::{
    BACKLINK_CHANNEL,
    BacklinkPayload,
    INCOMING_CHANNEL,
    IncomingPayload,
    LinkState,
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
        kv::{
            api::self_kv,
            types::Kv,
        },
        scene::{
            api::self_document,
            types::{
                Material,
                Portal,
                PortalDestination,
                PortalReceptor,
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
const LINK_KEY: &str = "gate:link";
const PORTAL_PRIM_NAME: &str = "portal";

const PORTAL_WIDTH: f32 = GOLDEN_RATIO;
const PORTAL_HEIGHT: f32 = PORTAL_WIDTH * GOLDEN_RATIO;

const BEAM_THICKNESS: f32 = 1.0 / (4.0 * GOLDEN_RATIO);

const PEDESTAL_HEIGHT: f32 = PORTAL_WIDTH / 2.0;
const PEDESTAL_THICKNESS: f32 = BEAM_THICKNESS * GOLDEN_RATIO;
const EVENT_RADIUS: f32 = PEDESTAL_THICKNESS;

struct Script {
    portal_prim: Prim,
    kv:          Kv,
    beacon_rx:   EventReceptor,
    incoming_rx: EventReceptor,
    backlink_rx: EventReceptor,
    applied:     Option<LinkState>,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = self_document()?;
        let root = doc.roots().into_iter().next().expect("root");

        // Authored in asset.hsdx so its TreeID is identical on every peer;
        // portal links carry that id and must resolve off-opener.
        let portal_prim = doc
            .prims()
            .into_iter()
            .find(|p| p.name().is_some_and(|n| n == PORTAL_PRIM_NAME))
            .context("portal prim not found")?;
        set_translation(&portal_prim, Vec3::new(0.0, PORTAL_HEIGHT / 2.0, 0.0));
        portal_prim.set_portal(Some(&portal_from_link(None)))?;

        let material = gate_material();
        spawn_frame(&root, &material);

        let pedestal_shape = Cuboid::new(Vec3::new(
            PEDESTAL_THICKNESS,
            PEDESTAL_HEIGHT,
            PEDESTAL_THICKNESS,
        ));

        let pedestal = pedestal_shape.mesh();
        root.add_child(&pedestal)?;
        pedestal.set_collider(Some(&pedestal_shape.collider()))?;
        pedestal.set_rigid_body(Some(static_body()))?;
        pedestal.set_material(Some(&material))?;
        set_translation(
            &pedestal,
            Vec3::new(-PORTAL_WIDTH, PEDESTAL_HEIGHT / 2.0, 0.0),
        );

        let receptor_prim = doc.create_prim()?;
        pedestal.add_child(&receptor_prim)?;
        set_translation(&receptor_prim, Vec3::new(0.0, PEDESTAL_HEIGHT / 2.0, 0.0));

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

        let incoming_rx = wired::event::api::listen(
            &[INCOMING_CHANNEL.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        )?;

        let backlink_rx = wired::event::api::listen(
            &[BACKLINK_CHANNEL.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        )?;

        println!("Gate ready");

        Ok(Self {
            portal_prim,
            kv: self_kv()?,
            beacon_rx,
            incoming_rx,
            backlink_rx,
            applied: None,
        })
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.beacon_rx.poll() {
            let payload = event.payload();
            let Ok(target) = <[u8; 32]>::try_from(payload.as_slice()) else {
                continue;
            };
            if read_link(&self.kv).is_some_and(|s| s.target_space == target) {
                continue;
            }
            write_link(
                &self.kv,
                &LinkState {
                    target_space:  target,
                    receptor_doc:  None,
                    receptor_prim: None,
                },
            );
            wired::portal::api::open(self.portal_prim.clone(), target.as_ref())?;
        }

        while let Some(event) = self.incoming_rx.poll() {
            if !event.consume() {
                continue;
            }
            let Ok(req) = postcard::from_bytes::<IncomingPayload>(&event.payload()) else {
                continue;
            };
            write_link(
                &self.kv,
                &LinkState {
                    target_space:  req.source_space,
                    receptor_doc:  Some(req.source_doc),
                    receptor_prim: Some(req.source_prim),
                },
            );
        }

        while let Some(event) = self.backlink_rx.poll() {
            let Ok(payload) = postcard::from_bytes::<BacklinkPayload>(&event.payload()) else {
                continue;
            };
            if payload.source_prim != self.portal_prim.id() {
                continue;
            }
            let Some(mut state) = read_link(&self.kv) else {
                continue;
            };
            let new_doc = Some(payload.receptor_doc);
            let new_prim = Some(payload.receptor_prim);
            if state.receptor_doc == new_doc && state.receptor_prim == new_prim {
                continue;
            }
            state.receptor_doc = new_doc;
            state.receptor_prim = new_prim;
            write_link(&self.kv, &state);
        }

        let next = read_link(&self.kv);
        if next != self.applied {
            self.portal_prim
                .set_portal(Some(&portal_from_link(next.as_ref())))?;
            self.applied = next;
        }
        Ok(())
    }
}

fn spawn_frame(root: &Prim, material: &Material) {
    let pole = Cuboid::new(Vec3::new(BEAM_THICKNESS, PORTAL_HEIGHT, BEAM_THICKNESS));

    let pole_l = pole.mesh();
    root.add_child(&pole_l).ok();
    pole_l.set_collider(Some(&pole.collider())).ok();
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
    pole_r.set_collider(Some(&pole.collider())).ok();
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
    beam_top.set_collider(Some(&beam.collider())).ok();
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
    prim.set_xform(Some(Xform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }))
    .ok();
}

const fn gate_material() -> Material {
    Material {
        alpha_cutoff:               None,
        alpha_mode:                 None,
        base_color:                 Some(Color {
            r: 0.7,
            g: 0.72,
            b: 0.78,
            a: 1.0,
        }),
        base_color_texture:         None,
        double_sided:               None,
        emissive:                   None,
        emissive_texture:           None,
        metallic:                   Some(0.6),
        metallic_roughness_texture: None,
        normal_texture:             None,
        occlusion_texture:          None,
        roughness:                  Some(0.4),
    }
}

fn portal_from_link(link: Option<&LinkState>) -> Portal {
    Portal {
        destination: link.map(|s| PortalDestination {
            space:    s.target_space.to_vec(),
            receptor: s
                .receptor_doc
                .zip(s.receptor_prim.clone())
                .map(|(d, p)| PortalReceptor {
                    document: d.to_vec(),
                    prim:     p,
                }),
        }),
        size_x:      PORTAL_WIDTH,
        size_y:      PORTAL_HEIGHT,
    }
}

fn write_link(kv: &Kv, state: &LinkState) {
    let bytes = postcard::to_allocvec(state).expect("encode link state");
    if let Err(err) = kv.set(LINK_KEY, &bytes) {
        eprintln!("Gate kv write failed: {err:?}");
    }
}

fn read_link(kv: &Kv) -> Option<LinkState> {
    kv.get(LINK_KEY)
        .and_then(|b| postcard::from_bytes::<LinkState>(&b).ok())
}
