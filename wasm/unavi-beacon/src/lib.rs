use std::{
    cell::RefCell,
    str::FromStr,
    time::{Duration, Instant},
};

use blake3::Hash;
use wired_prelude::prelude::*;

use crate::{
    unavi::{beacon_protocol::api::BeaconEmitter, shapes::api::Cuboid},
    wired::scene::{context::self_document, types::RigidBodyKind},
};

mod color;

wired_prelude::generate_script!(Script);

const SIZE: f32 = 0.15;
const EMIT_INTERVAL: Duration = Duration::from_secs(4);

struct Script {
    emitter: BeaconEmitter,
    time: RefCell<Instant>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let Some(node) = doc.nodes().into_iter().next() else {
            eprintln!("beacon error: no node");
            return Self {
                emitter: BeaconEmitter::new(&[]),
                time: RefCell::new(Instant::now()),
            };
        };
        let Ok(id) = Hash::from_str(&node.name().unwrap_or_default()) else {
            eprintln!("beacon error: invalid node name");
            return Self {
                emitter: BeaconEmitter::new(&[]),
                time: RefCell::new(Instant::now()),
            };
        };

        let cuboid = Cuboid::new(Vec3::splat(SIZE));

        node.set_mesh(Some(&cuboid.mesh()));
        node.set_collider(Some(&cuboid.collider()));
        node.set_rigid_body(Some(RigidBodyKind::Dynamic));

        let mat = doc.create_material();
        node.set_material(Some(&mat));

        let color = color::generate_beacon_color(id);
        mat.set_base_color(color);
        mat.set_roughness(0.7);
        mat.set_metallic(0.3);

        println!("beacon initialized: {id}");

        Self {
            emitter: BeaconEmitter::new(id.as_bytes()),
            time: RefCell::new(Instant::now()),
        }
    }

    fn tick(&self) {
        if self.time.borrow().elapsed() < EMIT_INTERVAL {
            return;
        }
        *self.time.borrow_mut() = Instant::now();
        self.emitter.emit();
    }

    fn render(&self) {}

    fn drop(&self) {}
}
