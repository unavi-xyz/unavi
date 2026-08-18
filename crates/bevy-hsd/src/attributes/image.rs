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
use hsd::attributes::{
    Attribute,
    image::{
        AddressMode,
        FilterMode,
        ImageAttr,
    },
    slots,
};
use image::GenericImageView;

use crate::{
    HsdSlots,
    attributes::{
        AttributeParser,
        ParseError,
    },
};

const MAX_TEXTURE_DIMS: u32 = 8192;
/// Ceiling on what one decode may allocate, sized to hold the largest texture
/// the dimension cap admits.
const MAX_DECODE_BYTES: u64 = 4 * (MAX_TEXTURE_DIMS as u64) * (MAX_TEXTURE_DIMS as u64);

#[derive(Component, Debug, Clone, Copy)]
pub struct ImageData(pub ImageAttr);

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
                commands.entity(prim).remove::<(ImageData, HsdImage)>();
            }
        }
        Ok(())
    }
}

pub fn rebuild_image(
    changed: Query<(Entity, &ImageData, &HsdSlots), Or<(Changed<ImageData>, Changed<HsdSlots>)>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for (prim, image, slots) in &changed {
        let Some(bytes) = slots.0.get(slots::IMAGE_DATA) else {
            continue;
        };

        let attr = &image.0;
        let mut sampler = ImageSamplerDescriptor::default();
        for (value, target) in [
            (attr.address_mode_u, &mut sampler.address_mode_u),
            (attr.address_mode_v, &mut sampler.address_mode_v),
            (attr.address_mode_w, &mut sampler.address_mode_w),
        ] {
            if let Some(v) = value {
                *target = address_mode(v);
            }
        }
        for (value, target) in [
            (attr.mag_filter, &mut sampler.mag_filter),
            (attr.min_filter, &mut sampler.min_filter),
            (attr.mipmap_filter, &mut sampler.mipmap_filter),
        ] {
            if let Some(v) = value {
                *target = filter_mode(v);
            }
        }

        let dyn_img = match decode(bytes) {
            Ok(img) => img,
            Err(err) => {
                warn!("failed to decode image: {err}");
                continue;
            }
        };

        let handle = image_assets.add(build_img(dyn_img, sampler, attr.srgb));
        commands.entity(prim).insert(HsdImage(handle));
    }
}

/// Decodes with the dimensions bounded up front.
///
/// A decompression bomb declares its dimensions in a few header bytes; the
/// limit must reach the decoder, since checking `dimensions()` after a plain
/// load checks an allocation that already happened.
fn decode(bytes: &[u8]) -> Result<image::DynamicImage, image::ImageError> {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_TEXTURE_DIMS);
    limits.max_image_height = Some(MAX_TEXTURE_DIMS);
    limits.max_alloc = Some(MAX_DECODE_BYTES);

    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format()?;
    reader.limits(limits);
    reader.decode()
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

const fn address_mode(mode: AddressMode) -> ImageAddressMode {
    match mode {
        AddressMode::Repeat => ImageAddressMode::Repeat,
        AddressMode::MirrorRepeat => ImageAddressMode::MirrorRepeat,
        AddressMode::ClampToEdge => ImageAddressMode::ClampToEdge,
    }
}

const fn filter_mode(mode: FilterMode) -> ImageFilterMode {
    match mode {
        FilterMode::Linear => ImageFilterMode::Linear,
        FilterMode::Nearest => ImageFilterMode::Nearest,
    }
}
