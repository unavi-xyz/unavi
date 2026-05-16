use std::{collections::BTreeMap, io::Cursor};

use bevy::{pbr::MeshMaterial3d, prelude::*};
use bevy_hsd::attributes::{image::HsdImage, material::HsdMaterial};
use hsd::{
    HSD_CONTAINER_ID, PrimMeta,
    attributes::{
        Attribute, Attributes, attributes_map,
        image::ImageAttr,
        material::{ColorVec, MaterialAttr},
    },
};
use image::{ImageFormat, RgbaImage};
use lorosurgeon::{ByteArray, MaybeMissing, Reconcile, reconcile::RootReconciler};
use rstest::rstest;
use tracing_test::traced_test;

use crate::common::*;

mod common;

#[traced_test]
#[rstest]
fn test_material_lifecycle(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);
    let root = tree.create(None).expect("create");
    let meta = tree.get_meta(root).expect("get meta");

    let attr = MaterialAttr {
        base_color: MaybeMissing::Present(ColorVec(vec![0.5, 0.1, 0.2, 1.0])),
        alpha_mode: MaybeMissing::Present("Blend".to_string()),
        metallic: MaybeMissing::Present(0.7),
        roughness: MaybeMissing::Present(0.3),
        ..Default::default()
    };
    reconcile_prim_material(&meta, attr);

    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<(&HsdMaterial, &MeshMaterial3d<StandardMaterial>)>();
    let (hsd_mat, mesh_mat) = q.iter(world).next().expect("material on prim");
    assert_eq!(hsd_mat.0, mesh_mat.0);

    let handle = hsd_mat.0.clone();
    let assets = ctx.app.world().resource::<Assets<StandardMaterial>>();
    let mat = assets.get(&handle).expect("standard material asset");
    assert_eq!(mat.alpha_mode, AlphaMode::Blend);
    assert!((mat.metallic - 0.7).abs() < 1e-5);
    assert!((mat.perceptual_roughness - 0.3).abs() < 1e-5);
    let LinearRgba {
        red,
        green,
        blue,
        alpha,
    } = mat.base_color.to_linear();
    assert!((red - 0.5).abs() < 1e-5);
    assert!((green - 0.1).abs() < 1e-5);
    assert!((blue - 0.2).abs() < 1e-5);
    assert!((alpha - 1.0).abs() < 1e-5);

    let attrs = attributes_map(&meta).expect("attributes map");
    attrs.delete(MaterialAttr::KEY).expect("delete");
    ctx.doc.commit();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&HsdMaterial>();
    assert!(q.iter(world).next().is_none());
}

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
    let blob_hash = ctx.upload_blob(png);

    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);

    let image_prim = tree.create(None).expect("create image");
    let image_meta = tree.get_meta(image_prim).expect("image meta");
    let image_attr = ImageAttr {
        address_mode_u: MaybeMissing::Missing,
        address_mode_v: MaybeMissing::Missing,
        address_mode_w: MaybeMissing::Missing,
        data: ByteArray::<32>::new(*blob_hash.as_bytes()),
        mag_filter: MaybeMissing::Missing,
        min_filter: MaybeMissing::Missing,
        mipmap_filter: MaybeMissing::Missing,
        name: MaybeMissing::Missing,
        srgb: MaybeMissing::Present(true),
    };
    reconcile_prim(
        &image_meta,
        Attributes {
            image: MaybeMissing::Present(image_attr),
            ..Default::default()
        },
        None,
    );

    let material_prim = tree.create(None).expect("create material");
    let material_meta = tree.get_meta(material_prim).expect("material meta");
    let material_attr = MaterialAttr {
        base_color_texture: MaybeMissing::Present(image_prim.to_string()),
        ..Default::default()
    };
    reconcile_prim(
        &material_meta,
        Attributes {
            material: MaybeMissing::Present(material_attr),
            ..Default::default()
        },
        None,
    );

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

#[traced_test]
#[rstest]
fn test_material_relationship(mut ctx: TestContext) {
    let tree = ctx.doc.get_tree(&*HSD_CONTAINER_ID);

    let prim_a = tree.create(None).expect("create a");
    let meta_a = tree.get_meta(prim_a).expect("meta a");
    let attr_a = MaterialAttr {
        base_color: MaybeMissing::Present(ColorVec(vec![1.0, 0.0, 0.0, 1.0])),
        ..Default::default()
    };
    reconcile_prim(
        &meta_a,
        Attributes {
            material: MaybeMissing::Present(attr_a),
            ..Default::default()
        },
        None,
    );

    let prim_b = tree.create(None).expect("create b");
    let meta_b = tree.get_meta(prim_b).expect("meta b");
    reconcile_relationship_only(
        &meta_b,
        BTreeMap::from([("material".to_string(), prim_a.to_string())]),
    );

    ctx.doc.commit();
    ctx.app.update();
    ctx.app.update();

    let world = ctx.app.world_mut();
    let mut q = world.query::<&HsdMaterial>();
    let handles: Vec<Handle<StandardMaterial>> = q.iter(world).map(|m| m.0.clone()).collect();
    assert_eq!(handles.len(), 2);
    assert_eq!(handles[0], handles[1], "B should share A's material handle");
}

fn reconcile_relationship_only(meta: &loro::LoroMap, relationships: BTreeMap<String, String>) {
    let prim = PrimMeta {
        attributes: MaybeMissing::Missing,
        relationships: MaybeMissing::Present(relationships),
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile");
}

fn reconcile_prim_material(meta: &loro::LoroMap, attr: MaterialAttr) {
    reconcile_prim(
        meta,
        Attributes {
            material: MaybeMissing::Present(attr),
            ..Default::default()
        },
        None,
    );
}

fn reconcile_prim(
    meta: &loro::LoroMap,
    attributes: Attributes,
    relationships: Option<BTreeMap<String, String>>,
) {
    let prim = PrimMeta {
        attributes: MaybeMissing::Present(attributes),
        relationships: relationships.map_or(MaybeMissing::Missing, MaybeMissing::Present),
    };
    prim.reconcile(RootReconciler::new(meta.clone()))
        .expect("reconcile");
}
