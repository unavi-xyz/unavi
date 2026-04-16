use std::str::FromStr;

use blake3::Hash;
use wired_prelude::prelude::*;

use crate::{
    unavi::shapes::api::Cuboid,
    wired::scene::{context::self_document, types::RigidBodyKind},
};

wired_prelude::generate_script!(Script);

const SIZE: f32 = 0.15;

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        let doc = self_document();
        let Some(node) = doc.nodes().into_iter().next() else {
            eprintln!("beacon error: no node");
            return Self;
        };
        let Ok(id) = Hash::from_str(&node.name().unwrap_or_default()) else {
            eprintln!("beacon error: invalid node name");
            return Self;
        };

        let cuboid = Cuboid::new(Vec3::splat(SIZE));

        node.set_mesh(Some(&cuboid.mesh()));
        node.set_collider(Some(&cuboid.collider()));
        node.set_rigid_body(Some(RigidBodyKind::Dynamic));

        let mat = doc.create_material();
        node.set_material(Some(&mat));

        let color = generate_beacon_color(id);
        mat.set_base_color(color);
        mat.set_roughness(0.7);
        mat.set_metallic(0.3);

        println!("beacon initialized: {id}");

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}

fn generate_beacon_color(hash: Hash) -> Color {
    let bytes = hash.as_slice();

    // Use first 8 bytes for hue (full spectrum)
    let hue_u64 = u64::from_le_bytes(bytes[0..8].try_into().expect("u64"));
    let h = (hue_u64 as f64 / u64::MAX as f64) as f32; // 0..1

    // Next bytes for saturation/value but clamp to nice ranges
    let s_u16 = u16::from_le_bytes(bytes[8..10].try_into().expect("u16"));
    let v_u16 = u16::from_le_bytes(bytes[10..12].try_into().expect("u16"));

    // Keep colors vivid but not neon / washed out
    let s = (f32::from(s_u16) / f32::from(u16::MAX)).mul_add(0.35, 0.55); // 0.55–0.9
    let v = (f32::from(v_u16) / f32::from(u16::MAX)).mul_add(0.30, 0.65); // 0.65–0.95

    Color::hsv(h, s, v)
}
