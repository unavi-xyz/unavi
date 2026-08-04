use std::io::Cursor;

use bevy::{
    image::ImageSampler,
    prelude::*,
    render::render_resource::TextureFormat,
};
use bevy_hsd::attributes::image::HsdImage;
use hsd::attributes::{
    image::ImageAttr,
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
fn test_image_lifecycle(mut ctx: TestContext) {
    let root = ctx.create_prim();
    ctx.set_attr(root, &ImageAttr::default());
    ctx.set_bulk(root, slots::IMAGE_DATA, blake3::hash(b"png"), 3);

    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut query = world.query::<&HsdImage>();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].0, Handle::<Image>::default());

    ctx.remove_attr::<ImageAttr>(root);
    ctx.app.update();

    let world = ctx.app.world_mut();
    let res = query.query(world).into_iter().collect::<Vec<_>>();
    assert!(res.is_empty());
}

#[traced_test]
#[rstest]
fn test_image_blob_load(#[from(ctx_wds)] mut ctx: TestContext) {
    let mut rgba = RgbaImage::new(2, 2);
    rgba.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    rgba.put_pixel(1, 0, image::Rgba([0, 255, 0, 255]));
    rgba.put_pixel(0, 1, image::Rgba([0, 0, 255, 255]));
    rgba.put_pixel(1, 1, image::Rgba([255, 255, 0, 255]));

    let mut png_bytes = Vec::new();
    image::DynamicImage::ImageRgba8(rgba.clone())
        .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .expect("encode png");

    let size = png_bytes.len() as u64;
    let data_hash = ctx.upload_blob(png_bytes);

    let root = ctx.create_prim();
    ctx.set_attr(
        root,
        &ImageAttr {
            address_mode_u: Some(1),
            mag_filter: Some(1),
            srgb: Some(true),
            ..Default::default()
        },
    );
    ctx.set_bulk(root, slots::IMAGE_DATA, data_hash, size);

    let mut handle: Option<Handle<Image>> = None;
    ctx.tick_until(|world| {
        let mut q = world.query::<&HsdImage>();
        let Some(h) = q.iter(world).next() else {
            return false;
        };
        if h.0 != Handle::<Image>::default() {
            handle = Some(h.0.clone());
            return true;
        }
        false
    });

    let handle = handle.expect("image handle");
    let assets = ctx.app.world().resource::<Assets<Image>>();
    let img = assets.get(&handle).expect("image asset");

    assert_eq!(img.width(), 2);
    assert_eq!(img.height(), 2);
    assert_eq!(img.texture_descriptor.format, TextureFormat::Rgba8UnormSrgb);
    assert_eq!(img.data.as_deref(), Some(rgba.as_raw().as_slice()));

    let ImageSampler::Descriptor(sampler) = &img.sampler else {
        panic!("expected descriptor sampler");
    };
    assert_eq!(
        sampler.address_mode_u,
        bevy::image::ImageAddressMode::MirrorRepeat
    );
    assert_eq!(sampler.mag_filter, bevy::image::ImageFilterMode::Nearest);
}
