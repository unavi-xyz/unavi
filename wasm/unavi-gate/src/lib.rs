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
        scene::{api::self_document, types::RigidBodyKind},
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

struct Script {
    receptor: EventReceptor,
    target: Option<(Hash, SystemTime)>,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();
        let root = doc.roots().into_iter().next().expect("root");

        let pole = Cuboid::new(Vec3::new(BEAM_THICKNESS, PORTAL_HEIGHT, BEAM_THICKNESS));

        let node_l = doc.create_node();
        root.add_child(&node_l);
        node_l.set_mesh(Some(&pole.mesh()));
        node_l.set_collider(Some(&pole.collider()));
        node_l.set_rigid_body(Some(RigidBodyKind::Fixed));
        node_l.set_translation(Vec3::new(
            -PORTAL_WIDTH / 2.0 - BEAM_THICKNESS / 2.0,
            PORTAL_HEIGHT / 2.0,
            0.0,
        ));

        let node_r = doc.create_node();
        root.add_child(&node_r);
        node_r.set_mesh(Some(&pole.mesh()));
        node_r.set_collider(Some(&pole.collider()));
        node_r.set_rigid_body(Some(RigidBodyKind::Fixed));
        node_r.set_translation(Vec3::new(
            PORTAL_WIDTH / 2.0 + BEAM_THICKNESS / 2.0,
            PORTAL_HEIGHT / 2.0,
            0.0,
        ));

        let beam = Cuboid::new(Vec3::new(
            BEAM_THICKNESS.mul_add(2.0, PORTAL_WIDTH),
            BEAM_THICKNESS,
            BEAM_THICKNESS,
        ));

        let node_t = doc.create_node();
        root.add_child(&node_t);
        node_t.set_mesh(Some(&beam.mesh()));
        node_t.set_collider(Some(&beam.collider()));
        node_t.set_rigid_body(Some(RigidBodyKind::Fixed));
        node_t.set_translation(Vec3::new(0.0, PORTAL_HEIGHT + BEAM_THICKNESS / 2.0, 0.0));

        let pedestal_shape = Cuboid::new(Vec3::new(
            PEDESTAL_THICKNESS,
            PEDESTAL_HEIGHT,
            PEDESTAL_THICKNESS,
        ));

        let pedestal = doc.create_node();
        root.add_child(&pedestal);
        pedestal.set_mesh(Some(&pedestal_shape.mesh()));
        pedestal.set_collider(Some(&pedestal_shape.collider()));
        pedestal.set_rigid_body(Some(RigidBodyKind::Fixed));
        pedestal.set_translation(Vec3::new(-PORTAL_WIDTH, PEDESTAL_HEIGHT / 2.0, 0.0));

        let receptor_node = doc.create_node();
        pedestal.add_child(&receptor_node);
        receptor_node.set_translation(Vec3::new(0.0, PEDESTAL_HEIGHT, 0.0));

        let receptor = wired::event::api::listen(
            &[CHANNEL.to_string()],
            EventFilter {
                documents: None,
                scope: EventScope::Spatial(SpatialScope {
                    node: receptor_node,
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
