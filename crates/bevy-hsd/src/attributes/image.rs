use bevy::{
    asset::RenderAssetUsages,
    image::{
        ImageAddressMode,
        ImageFilterMode,
        ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
    render::render_resource::{
        Extent3d,
        TextureDimension,
        TextureFormat,
    },
};
use bevy_wds::blob::{
    deps::{
        BlobDep,
        BlobDeps,
        BlobDepsLoaded,
    },
    request::{
        BlobRequest,
        BlobResponse,
    },
};
use hsd::attributes::{
    Attribute,
    image::ImageAttr,
    slots,
};
use image::GenericImageView;

use crate::{
    HsdBulk,
    attributes::{
        AttributeParser,
        ParseError,
    },
};

const MAX_TEXTURE_DIMS: u32 = 8192;

#[derive(Component, Debug, Clone, Copy)]
pub struct ImageData(pub ImageAttr);

#[derive(Component)]
#[require(BlobDeps)]
pub struct ImageBlobs {
    pub data:    Entity,
    pub sampler: ImageSamplerDescriptor,
    pub srgb:    Option<bool>,
}

#[derive(Component)]
#[relationship(relationship_target = ImageBlobsChild)]
pub struct ImageBlobsOwner(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = ImageBlobsOwner, linked_spawn)]
pub struct ImageBlobsChild(Entity);

#[derive(Component, Default)]
pub struct HsdImage(pub Handle<Image>);

pub struct ImageParser;

impl AttributeParser for ImageParser {
    fn key(&self) -> &'static str {
        ImageAttr::KEY
    }

    fn lifecycle(
        &self,
        commands: &mut Commands,
        prim: Entity,
        payload: Option<&[u8]>,
    ) -> Result<(), ParseError> {
        match payload {
            Some(payload) => {
                commands
                    .entity(prim)
                    .insert((ImageData(ImageAttr::decode(payload)?), HsdImage::default()));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<(ImageData, HsdImage, ImageBlobsChild)>();
            }
        }
        Ok(())
    }
}

pub fn rebuild_image(
    changed: Query<(Entity, &ImageData, &HsdBulk), Or<(Changed<ImageData>, Changed<HsdBulk>)>>,
    mut commands: Commands,
) {
    for (prim, image, bulk) in &changed {
        let Some(hash) = bulk.0.get(slots::IMAGE_DATA) else {
            continue;
        };

        commands.entity(prim).remove::<ImageBlobsChild>();

        let child = commands.spawn(ImageBlobsOwner(prim)).id();
        let data = commands
            .spawn((
                BlobDep(child),
                BlobRequest(blake3::Hash::from_bytes(hash.0)),
            ))
            .id();

        let attr = &image.0;
        let mut sampler = ImageSamplerDescriptor::default();
        if let Some(v) = attr.address_mode_u {
            sampler.address_mode_u = address_mode(v);
        }
        if let Some(v) = attr.address_mode_v {
            sampler.address_mode_v = address_mode(v);
        }
        if let Some(v) = attr.address_mode_w {
            sampler.address_mode_w = address_mode(v);
        }
        if let Some(v) = attr.mag_filter {
            sampler.mag_filter = filter_mode(v);
        }
        if let Some(v) = attr.min_filter {
            sampler.min_filter = filter_mode(v);
        }
        if let Some(v) = attr.mipmap_filter {
            sampler.mipmap_filter = filter_mode(v);
        }

        commands.entity(child).insert(ImageBlobs {
            data,
            sampler,
            srgb: attr.srgb,
        });
    }
}

pub fn on_image_blob_loaded(
    trigger: On<Add, BlobDepsLoaded>,
    image_blobs: Query<(&ImageBlobs, &ImageBlobsOwner)>,
    mut blob_responses: Query<&mut BlobResponse>,
    mut hsd_images: Query<&mut HsdImage>,
    mut image_assets: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let child = trigger.entity;
    let Ok((params, owner)) = image_blobs.get(child) else {
        return;
    };
    let prim = owner.0;

    let Ok(Some(bytes)) = blob_responses.get_mut(params.data).map(|mut b| b.0.take()) else {
        warn!("image blob not found");
        commands.entity(child).try_despawn();
        return;
    };

    let dyn_img = match image::load_from_memory(&bytes) {
        Ok(img) => img,
        Err(err) => {
            warn!(?err, "failed to decode image");
            commands.entity(child).try_despawn();
            return;
        }
    };

    let img = build_img(dyn_img, params.sampler.clone(), params.srgb);
    let handle = image_assets.add(img);

    if let Ok(mut hsd_image) = hsd_images.get_mut(prim) {
        hsd_image.0 = handle;
    } else {
        commands.entity(prim).insert(HsdImage(handle));
    }

    commands.entity(child).try_despawn();
}

fn build_img(
    dyn_img: image::DynamicImage,
    sampler: ImageSamplerDescriptor,
    srgb: Option<bool>,
) -> Image {
    let (width, height) = dyn_img.dimensions();
    if width > MAX_TEXTURE_DIMS || height > MAX_TEXTURE_DIMS {
        warn!("image too large: {width}x{height}");
        return Image::default();
    }

    let rgba = dyn_img.into_rgba8();
    let format = if srgb == Some(false) {
        TextureFormat::Rgba8Unorm
    } else {
        TextureFormat::Rgba8UnormSrgb
    };

    let mut img = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        format,
        RenderAssetUsages::default(),
    );

    img.sampler = ImageSampler::Descriptor(sampler);

    img
}

const fn address_mode(v: i64) -> ImageAddressMode {
    match v {
        1 => ImageAddressMode::MirrorRepeat,
        2 => ImageAddressMode::ClampToEdge,
        _ => ImageAddressMode::Repeat,
    }
}

const fn filter_mode(v: i64) -> ImageFilterMode {
    match v {
        1 => ImageFilterMode::Nearest,
        _ => ImageFilterMode::Linear,
    }
}
