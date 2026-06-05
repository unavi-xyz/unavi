use std::time::SystemTime;

use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        agent::{
            api::local_agent,
            types::BoneName,
        },
        scene::types::{
            Material,
            Prim,
            Xform,
        },
    },
};

wired_prelude::generate_script!(Script);

struct Script {
    hand: Prim,
    prim: Prim,
    time: SystemTime,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let size = 0.1;
        let prim = Cuboid::new(Vec3::splat(size)).mesh();

        prim.set_material(Some(&Material {
            alpha_cutoff:               None,
            alpha_mode:                 None,
            base_color:                 Some(Color::rgba(0.8, 0.1, 0.1, 1.0)),
            base_color_texture:         None,
            double_sided:               None,
            emissive:                   None,
            emissive_texture:           None,
            metallic:                   None,
            metallic_roughness_texture: None,
            normal_texture:             None,
            occlusion_texture:          None,
            roughness:                  None,
        }));

        let agent = local_agent()?;
        let hand = agent.bone(BoneName::RightHand).expect("get bone");

        Ok(Self {
            hand,
            prim,
            time: SystemTime::now(),
        })
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        let now = self.time.elapsed().expect("elapsed").as_secs_f32();
        let offset = Vec3::new(0.0, now.sin() * 0.1, 0.0);

        let global = self.hand.global_xform();
        let mut tr = global.translation;
        tr.x += offset.x;
        tr.y += offset.y;
        tr.z += offset.z;
        self.prim.set_xform(Some(Xform {
            translation: tr,
            rotation:    global.rotation,
            scale:       global.scale,
        }));
        Ok(())
    }
}
