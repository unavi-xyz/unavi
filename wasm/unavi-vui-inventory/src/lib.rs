use wired_prelude::prelude::*;

use crate::{
    unavi::{
        shapes::api::Cuboid,
        vui_module::api::{ModuleEvent, VuiModule},
    },
    wired::scene::{
        api::self_document,
        types::{Material, Prim, RigidBody, RigidBodyKind, Xform},
    },
};

wired_prelude::generate_script!(Script);

const NAME: &str = "Inventory";

const BASE_H: f32 = 0.016;
const ICON_SIZE: f32 = 0.040;
const LIP_H: f32 = 0.036;
const LIP_T: f32 = 0.012;
const LIP_Y: f32 = BASE_H * 0.5 + LIP_H * 0.5;
const TABLE_D: f32 = 0.44;
const TABLE_W: f32 = 0.60;
const X_LIP_X: f32 = TABLE_W * 0.5 - LIP_T * 0.5;
const Z_LIP_Z: f32 = TABLE_D * 0.5 - LIP_T * 0.5;

const IDENTITY_QUAT: Quat = Quat {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 1.0,
};

fn set_translation(prim: &Prim, translation: Vec3) {
    prim.set_xform(Some(Xform {
        translation,
        rotation: IDENTITY_QUAT,
        scale: Vec3::splat(1.0),
    }));
}

fn set_scale(prim: &Prim, scale: Vec3) {
    prim.set_xform(Some(Xform {
        translation: Vec3::splat(0.0),
        rotation: IDENTITY_QUAT,
        scale,
    }));
}

const fn material(base_color: Option<Color>) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode: None,
        base_color,
        base_color_texture: None,
        double_sided: Some(true),
        emissive: None,
        emissive_texture: None,
        metallic: None,
        metallic_roughness_texture: None,
        normal_texture: None,
        occlusion_texture: None,
        roughness: None,
    }
}

const fn static_body() -> RigidBody {
    RigidBody {
        kind: RigidBodyKind::Static,
        angular_damping: None,
        friction: None,
        linear_damping: None,
        mass: None,
        restitution: None,
    }
}

struct Script {
    root: Prim,
    _icon: Prim,
    module: VuiModule,
    color: Color,
    themed_prims: Vec<Prim>,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let doc = self_document();

        let color = Color::WHITE;
        let color_mat = material(Some(color));

        let root = doc.create_prim();
        set_scale(&root, Vec3::splat(0.0));

        let mut themed_prims = Vec::new();

        let base_shape = Cuboid::new(Vec3::new(TABLE_W, BASE_H, TABLE_D));
        let base = base_shape.mesh();
        base.set_collider(Some(&base_shape.collider()));
        base.set_rigid_body(Some(static_body()));
        base.set_material(Some(&color_mat));
        root.add_child(&base);
        themed_prims.push(base);

        let x_lip_shape = Cuboid::new(Vec3::new(LIP_T, LIP_H, TABLE_D));
        for x_sign in [-1.0_f32, 1.0_f32] {
            let lip = x_lip_shape.mesh();
            lip.set_collider(Some(&x_lip_shape.collider()));
            lip.set_rigid_body(Some(static_body()));
            lip.set_material(Some(&color_mat));
            set_translation(&lip, Vec3::new(x_sign * X_LIP_X, LIP_Y, 0.0));
            root.add_child(&lip);
            themed_prims.push(lip);
        }

        let z_lip_shape = Cuboid::new(Vec3::new(TABLE_W, LIP_H, LIP_T));
        for z_sign in [-1.0_f32, 1.0_f32] {
            let lip = z_lip_shape.mesh();
            lip.set_collider(Some(&z_lip_shape.collider()));
            lip.set_rigid_body(Some(static_body()));
            lip.set_material(Some(&color_mat));
            set_translation(&lip, Vec3::new(0.0, LIP_Y, z_sign * Z_LIP_Z));
            root.add_child(&lip);
            themed_prims.push(lip);
        }

        let icon = Cuboid::new(Vec3::splat(ICON_SIZE)).mesh();
        let module = VuiModule::new(NAME, &icon);

        Self {
            root,
            _icon: icon,
            module,
            color,
            themed_prims,
        }
    }

    fn tick(&mut self) {
        while let Some(event) = self.module.poll() {
            match event {
                ModuleEvent::Activate(t) => {
                    self.root.set_xform(Some(Xform {
                        translation: t.translation,
                        rotation: t.rotation,
                        scale: t.scale,
                    }));
                }
                ModuleEvent::Deactivate => {
                    set_scale(&self.root, Vec3::splat(0.0));
                }
                ModuleEvent::SetColor(color) => {
                    self.color = color;
                    let mat = material(Some(color));
                    for prim in &self.themed_prims {
                        prim.set_material(Some(&mat));
                    }
                }
            }
        }
    }
}
