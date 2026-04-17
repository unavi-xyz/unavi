use std::{
    cell::RefCell,
    f32::consts::GOLDEN_RATIO,
    time::{Duration, Instant},
};

use blake3::Hash;
use wired_prelude::prelude::*;

use crate::{
    unavi::{beacon_protocol::api::BeaconReceptor, shapes::api::Cuboid},
    wired::scene::{context::self_document, types::RigidBodyKind},
};

wired_prelude::generate_script!(Script);

const PORTAL_WIDTH: f32 = 1.7;
const PORTAL_HEIGHT: f32 = PORTAL_WIDTH * GOLDEN_RATIO;

const BEAM_THICKNESS: f32 = 0.2;

const PEDESTAL_HEIGHT: f32 = 0.75;
const PEDESTAL_THICKNESS: f32 = BEAM_THICKNESS * 1.5;
const EVENT_RADIUS: f32 = PEDESTAL_THICKNESS;

const TARGET_DECAY: Duration = Duration::from_secs(10);

struct Script {
    receptor: BeaconReceptor,
    target: RefCell<Option<(Vec<u8>, Instant)>>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();

        let pole = Cuboid::new(Vec3::new(BEAM_THICKNESS, PORTAL_HEIGHT, BEAM_THICKNESS));

        let node_l = doc.create_node();
        node_l.set_mesh(Some(&pole.mesh()));
        node_l.set_collider(Some(&pole.collider()));
        node_l.set_rigid_body(Some(RigidBodyKind::Fixed));
        node_l.set_translation(Vec3::new(
            -PORTAL_WIDTH / 2.0 - BEAM_THICKNESS / 2.0,
            PORTAL_HEIGHT / 2.0,
            0.0,
        ));

        let node_r = doc.create_node();
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
        pedestal.set_mesh(Some(&pedestal_shape.mesh()));
        pedestal.set_collider(Some(&pedestal_shape.collider()));
        pedestal.set_rigid_body(Some(RigidBodyKind::Fixed));
        pedestal.set_translation(Vec3::new(-PORTAL_WIDTH, PEDESTAL_HEIGHT / 2.0, 0.0));

        let receptor_node = doc.create_node();
        receptor_node.set_translation(Vec3::new(0.0, PEDESTAL_HEIGHT, 0.0));
        pedestal.add_child(&receptor_node);

        Self {
            receptor: BeaconReceptor::new(receptor_node, EVENT_RADIUS),
            target: RefCell::new(None),
        }
    }

    fn tick(&self) {
        let mut target = self.target.borrow_mut();
        if let Some((_, t)) = &*target
            && t.elapsed() >= TARGET_DECAY
        {
            *target = None;
        }

        while let Some(id) = self.receptor.poll() {
            if target.is_none() || target.as_ref().is_some_and(|(x, _)| *x == id) {
                let Ok(id_hash) = Hash::from_slice(&id) else {
                    continue;
                };
                println!("loading beacon: {id_hash}");
                *target = Some((id, Instant::now()));
            }
        }
    }

    fn render(&self) {}

    fn drop(&self) {}
}
