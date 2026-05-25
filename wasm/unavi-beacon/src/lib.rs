use std::{
    str::FromStr,
    time::{
        Duration,
        SystemTime,
    },
};

use blake3::Hash;
use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        event::types::{
            EventFilter,
            EventScope,
            SpatialScope,
        },
        scene::{
            api::self_document,
            types::{
                Material,
                Prim,
                RigidBody,
                RigidBodyKind,
            },
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
    id:   Hash,
    prim: Prim,
    time: SystemTime,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();

        let Some((id, prim)) = doc.prims().into_iter().find_map(|p| {
            Hash::from_str(&p.name().unwrap_or_default())
                .ok()
                .map(|id| (id, p))
        }) else {
            panic!("invalid beacon: id prim not found")
        };

        let cuboid = Cuboid::new(Vec3::splat(SIZE));
        let cube = cuboid.mesh();
        cube.set_collider(Some(&cuboid.collider()));
        cube.set_rigid_body(Some(RigidBody {
            kind:            RigidBodyKind::Dynamic,
            angular_damping: None,
            friction:        None,
            linear_damping:  None,
            mass:            None,
            restitution:     None,
        }));
        prim.add_child(&cube);

        let color = color::generate_beacon_color(id);
        cube.set_material(Some(&Material {
            alpha_cutoff:               None,
            alpha_mode:                 None,
            base_color:                 Some(color),
            base_color_texture:         None,
            double_sided:               None,
            emissive:                   None,
            emissive_texture:           None,
            metallic:                   Some(0.3),
            metallic_roughness_texture: None,
            normal_texture:             None,
            occlusion_texture:          None,
            roughness:                  Some(0.7),
        }));
        println!("Beacon initialized: {id}");
        Self {
            id,
            prim,
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
                scope:     EventScope::Spatial(SpatialScope {
                    prim:   self.prim.clone(),
                    radius: EVENT_RADIUS,
                }),
            },
        );
    }
}
