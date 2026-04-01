use crate::wired::scene::{context::self_document, types::AlphaMode};
use wired_prelude::wired_scene::types::Color;

use crate::check;

pub fn test_material() {
    let doc = self_document();
    let mat = doc.create_material();

    check("mat id non-empty", !mat.id().is_empty(), true);

    mat.set_name(Some("test-mat"));
    check("mat name", mat.name().as_deref(), Some("test-mat"));

    mat.set_alpha_cutoff(0.3);
    check("mat alpha_cutoff", mat.alpha_cutoff(), 0.3_f32);

    mat.set_alpha_mode(Some(AlphaMode::Blend));
    check("mat alpha_mode", mat.alpha_mode(), Some(AlphaMode::Blend));

    let color = Color {
        r: 0.1,
        g: 0.2,
        b: 0.3,
        a: 1.0,
    };
    mat.set_base_color(color);
    let got = mat.base_color();
    check("mat base_color r", got.r, 0.1_f32);
    check("mat base_color g", got.g, 0.2_f32);
    check("mat base_color b", got.b, 0.3_f32);
    check("mat base_color a", got.a, 1.0_f32);

    mat.set_metallic(0.8);
    check("mat metallic", mat.metallic(), 0.8_f32);

    mat.set_roughness(0.4);
    check("mat roughness", mat.roughness(), 0.4_f32);

    mat.set_double_sided(true);
    check("mat double_sided", mat.double_sided(), true);

    mat.set_unlit(true);
    check("mat unlit", mat.unlit(), true);
}
