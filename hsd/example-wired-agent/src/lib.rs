//! Attaches a cube to the local agent's camera and a bone in turn, to show
//! `wired:agent/local.attach` carrying a document instead of a script
//! copying a tracked transform every frame.

use wired_guest::math::{
    Color,
    Transform,
    Vec3,
};

use crate::{
    unavi::shapes::api::Cuboid,
    wired::{
        agent::local::{
            Attachment,
            HumanoidBone,
        },
        scene::{
            document::{
                Document,
                script_document,
            },
            properties::{
                Material,
                Property,
            },
        },
    },
};

wired_guest::generate_script!(Script);

struct Script {
    doc: Document,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let doc = script_document()?;

        let size = 0.1;
        let prim = Cuboid::new(Vec3::splat(size)).mesh()?;

        doc.local()
            .set(
                prim,
                Property::Material(Material::solid(Color::rgba(0.8, 0.1, 0.1, 1.0))),
            )
            .flush()?;

        Ok(Self { doc })
    }

    fn fixed_update(
        &mut self,
        tick: exports::wired::script::lifecycle::Tick,
    ) -> anyhow::Result<()> {
        let offset = Vec3::new(0.0, (tick.time as f32).sin() * 0.1, 0.0);

        wired::agent::local::attach(
            &self.doc,
            Attachment::Bone(HumanoidBone::RightHand),
            Transform::from_translation(offset),
        )?;
        Ok(())
    }
}
