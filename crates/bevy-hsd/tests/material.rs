use std::io::Cursor;

use bevy::{
    pbr::MeshMaterial3d,
    prelude::*,
};
use bevy_hsd::attributes::{
    image::HsdImage,
    material::HsdMaterial,
};
use hsd::attributes::{
    image::ImageAttr,
    material::{
        self,
        ColorVec,
        MaterialAttr,
    },
    slots,
};
use image::{
    ImageFormat,
    RgbaImage,
};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_material_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();

    ctx.set_attr(
        root,
        &MaterialAttr {
            base_color: Some(ColorVec(vec![0.5, 0.1, 0.2, 1.0])),
            alpha_mode: Some("Blend".to_string()),
            metallic: Some(0.7),
            roughness: Some(0.3),
            ..Default::default()
        },
    );

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<(&HsdMaterial, &MeshMaterial3d<StandardMaterial>)>();
    let (hsd_mat, mesh_mat) = q.iter(world).next().expect("material on prim");
    assert_eq!(hsd_mat.0, mesh_mat.0);

    let handle = hsd_mat.0.clone();
    let assets = ctx.app.world().resource::<Assets<StandardMaterial>>();
    let mat = assets.get(&handle).expect("standard material asset");
    assert_eq!(mat.alpha_mode, AlphaMode::Blend);
    assert!((mat.metallic - 0.7).abs() < 1.0e-5);
    assert!((mat.perceptual_roughness - 0.3).abs() < 1.0e-5);
    let LinearRgba {
        red,
        green,
        blue,
        alpha,
    } = mat.base_color.to_linear();
    assert!((red - 0.5).abs() < 1.0e-5);
    assert!((green - 0.1).abs() < 1.0e-5);
    assert!((blue - 0.2).abs() < 1.0e-5);
    assert!((alpha - 1.0).abs() < 1.0e-5);

    ctx.remove_attr::<MaterialAttr>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&HsdMaterial>();
    assert!(q.iter(world).next().is_none());
}

/// A texture slot is a relationship, not a hash inside the material payload:
/// one property namespace, one home for a cross-prim reference.
#[traced_test]
#[rstest]
fn test_material_texture_ref(#[from(ctx_wds)] mut ctx: TestContext) {
    let mut rgba = RgbaImage::new(2, 2);
    for (i, px) in rgba.pixels_mut().enumerate() {
        let v = (i * 60) as u8;
        *px = image::Rgba([v, v, v, 255]);
    }
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .expect("encode png");

    let image_prim = ctx.create_prim();
    ctx.set_attr(
        image_prim,
        &ImageAttr {
            srgb: Some(true),
            ..Default::default()
        },
    );
    ctx.set_slot(image_prim, slots::IMAGE_DATA, png);

    let material_prim = ctx.create_prim();
    ctx.set_attr(material_prim, &MaterialAttr::default());
    ctx.set_relationship(material_prim, material::BASE_COLOR_TEXTURE, image_prim);

    let mut image_handle: Option<Handle<Image>> = None;
    let mut material_handle: Option<Handle<StandardMaterial>> = None;
    ctx.tick_until(|world| {
        let imgs: Vec<Handle<Image>> = world
            .query::<&HsdImage>()
            .iter(world)
            .map(|i| i.0.clone())
            .collect();
        let mats: Vec<Handle<StandardMaterial>> = world
            .query::<&HsdMaterial>()
            .iter(world)
            .map(|m| m.0.clone())
            .collect();
        let assets = world.resource::<Assets<StandardMaterial>>();

        let img = imgs.into_iter().find(|h| *h != Handle::<Image>::default());
        let Some(img) = img else {
            return false;
        };

        for handle in mats {
            let Some(sm) = assets.get(&handle) else {
                continue;
            };
            if sm.base_color_texture.as_ref() == Some(&img) {
                image_handle = Some(img);
                material_handle = Some(handle);
                return true;
            }
        }
        false
    });

    let image_handle = image_handle.expect("image handle");
    let material_handle = material_handle.expect("material handle");
    let assets = ctx.app.world().resource::<Assets<StandardMaterial>>();
    let mat = assets.get(&material_handle).expect("standard material");
    assert_eq!(mat.base_color_texture.as_ref(), Some(&image_handle));
}

/// `material:binding` is USD's precedent: a prim uses another prim's material
/// rather than defining its own.
#[traced_test]
#[rstest]
fn test_material_binding(mut ctx: TestContext) {
    let prim_a = ctx.create_prim();
    ctx.set_attr(
        prim_a,
        &MaterialAttr {
            base_color: Some(ColorVec(vec![1.0, 0.0, 0.0, 1.0])),
            ..Default::default()
        },
    );

    let prim_b = ctx.create_prim();
    ctx.set_relationship(prim_b, material::BINDING, prim_a);

    ctx.app.update();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&HsdMaterial>();
    let handles: Vec<Handle<StandardMaterial>> = q.iter(world).map(|m| m.0.clone()).collect();
    assert_eq!(handles.len(), 2);
    assert_eq!(handles[0], handles[1], "B should share A's material handle");
}
