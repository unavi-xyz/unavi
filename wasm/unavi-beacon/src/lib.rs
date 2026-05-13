use std::{
    str::FromStr,
    time::{Duration, SystemTime},
};

use blake3::Hash;
use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        event::types::{EventFilter, EventScope, SpatialScope},
        scene::{
            api::self_document,
            types::{Node, RigidBodyKind},
        },
    },
};

mod color;

wired_prelude::generate_script!(Script);

const CHANNEL: &str = "unavi::beacon::id";
const EMIT_INTERVAL: Duration = Duration::from_secs(3);
const EVENT_RADIUS: f32 = SIZE * 3.0;
const SIZE: f32 = 0.15;

struct Script {
    id: Hash,
    node: Node,
    time: SystemTime,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();
        let Some(node) = doc.nodes().into_iter().next() else {
            panic!("beacon error: no node")
        };
        let Ok(id) = Hash::from_str(&node.name().unwrap_or_default()) else {
            panic!("beacon error: invalid node name")
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

        println!("Beacon initialized: {id}");
        Self {
            id,
            node,
            time: SystemTime::now(),
        }
    }

    fn tick(&mut self) {
        if self.time.elapsed().expect("elapsed") < EMIT_INTERVAL {
            return;
        }
        self.time = SystemTime::now();

        println!("Emitting beacon event");
        wired::event::api::emit(
            CHANNEL,
            self.id.as_bytes(),
            EventFilter {
                documents: None,
                scope: EventScope::Spatial(SpatialScope {
                    node: self.node.clone(),
                    radius: EVENT_RADIUS,
                }),
            },
        );
    }
}
