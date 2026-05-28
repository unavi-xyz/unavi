use blake3::Hash;
use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{
        api::{
            load_hsd,
            self_document,
        },
        types::{
            Material,
            RigidBody,
            RigidBodyKind,
            Xform,
        },
    },
};

wired_prelude::generate_script!(Script);

const GROUND_SIZE: f32 = 30.0;
const GROUND_THICK: f32 = 0.5;

const IDENTITY_QUAT: Quat = Quat::IDENTITY;

fn set_translation(prim: &crate::wired::scene::types::Prim, translation: Vec3) {
    prim.set_xform(Some(Xform {
        translation,
        rotation: IDENTITY_QUAT,
        scale: Vec3::ONE,
    }));
}

struct Script;

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();

        let shape = Cuboid::new(Vec3::new(GROUND_SIZE, GROUND_THICK, GROUND_SIZE));

        let prim = shape.mesh();
        prim.set_collider(Some(&shape.collider()));
        prim.set_rigid_body(Some(RigidBody {
            kind:            RigidBodyKind::Static,
            angular_damping: None,
            friction:        None,
            linear_damping:  None,
            mass:            None,
            restitution:     None,
        }));
        set_translation(&prim, Vec3::new(0.0, -GROUND_THICK / 2.0, 0.0));

        let id = Hash::from_slice(&doc.id()).expect("document id");
        let base_color = unavi_script_util::color::generate_muted_color(id);
        prim.set_material(Some(&Material {
            alpha_cutoff:               None,
            alpha_mode:                 None,
            base_color:                 Some(base_color),
            base_color_texture:         None,
            double_sided:               None,
            emissive:                   None,
            emissive_texture:           None,
            metallic:                   Some(0.1),
            metallic_roughness_texture: None,
            normal_texture:             None,
            occlusion_texture:          None,
            roughness:                  Some(0.85),
        }));

        // Find a root prim named "gate" and use its asset blob id.
        let gate_blob = doc
            .roots()
            .into_iter()
            .find(|p| p.name().as_deref() == Some("gate"))
            .and_then(|p| p.asset());

        let Some(gate_blob) = gate_blob else {
            eprintln!("no gate asset");
            return Self;
        };
        let Ok(gate_doc) = load_hsd(&gate_blob) else {
            eprintln!("error loading gate HSD");
            return Self;
        };

        for root in gate_doc.roots() {
            set_translation(&root, Vec3::new(0.0, 0.0, -GROUND_SIZE / 3.0));
        }

        println!("Welcome home! =)");

        Self
    }
}
