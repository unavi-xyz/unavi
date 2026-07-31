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
                Material,
                Prim,
                RigidBody,
                RigidBodyKind,
            },
        },
    },
};

wired_prelude::generate_script!(Script);

const CHANNEL: &str = "unavi::beacon::id";
const EMIT_INTERVAL: Duration = Duration::from_secs(3);
const EVENT_RADIUS: f32 = SIZE * 3.0;
const SIZE: f32 = 0.15;

struct Script {
    cube:      Prim,
    id:        Hash,
    input:     InputListener,
    published: bool,
    time:      SystemTime,
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

        // A published beacon bakes its cube into the synced record, so a peer
        // that already holds it reuses that cube rather than authoring a copy.
        let cube = prim.children().into_iter().next().unwrap_or_else(|| {
            let cuboid = Cuboid::new(Vec3::splat(SIZE));
            let cube = cuboid.mesh();
            cube.set_collider(Some(&cuboid.collider())).ok();
            cube.set_rigid_body(Some(RigidBody {
                kind:            RigidBodyKind::Dynamic,
                angular_damping: None,
                friction:        None,
                linear_damping:  None,
                mass:            None,
                restitution:     None,
            }))
            .ok();
            cube.set_material(Some(&Material {
                alpha_cutoff:               None,
                alpha_mode:                 None,
                base_color:                 Some(unavi_script_util::color::generate_color(id)),
                base_color_texture:         None,
                double_sided:               None,
                emissive:                   None,
                emissive_texture:           None,
                metallic:                   Some(0.3),
                metallic_roughness_texture: None,
                normal_texture:             None,
                occlusion_texture:          None,
                roughness:                  Some(0.7),
            }))
            .ok();
            prim.add_child(&cube).ok();
            cube
        });

        let input = register_input_listener(&cube)?;
        println!("Beacon initialized: space={id}");
        Ok(Self {
            cube,
            id,
            input,
            published: false,
            time: SystemTime::now(),
        })
    }

    fn tick(&mut self) -> anyhow::Result<()> {
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

        if self.time.elapsed().expect("elapsed") < EMIT_INTERVAL {
            return Ok(());
        }
        self.time = SystemTime::now();

        wired::event::api::emit(
            CHANNEL,
            self.id.as_bytes(),
            EventFilter {
                documents: None,
                scope:     EventScope::Spatial(SpatialScope {
                    prim:   self.cube.clone(),
                    radius: EVENT_RADIUS,
                }),
            },
        )?;
        Ok(())
    }
}
