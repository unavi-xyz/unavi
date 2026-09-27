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
use hsd::{
    property::{
        Payload,
        name::PropName,
    },
    schema::image::{
        self as hsd_image,
        AddressMode,
        FilterMode,
    },
};
use image::GenericImageView;

use crate::attributes::{
    ParseError,
    Pending,
};

const MAX_TEXTURE_DIMS: u32 = 8192;
/// Ceiling on what one decode may allocate, sized to hold the largest texture
/// the dimension cap admits.
const MAX_DECODE_BYTES: u64 = 4 * (MAX_TEXTURE_DIMS as u64) * (MAX_TEXTURE_DIMS as u64);

/// The `image/sampler` field. Kept separately from the encoded bytes so a
/// sampler-only edit never re-decodes them.
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct HsdImageSampler(pub hsd_image::ImageSampler);

/// The `image/data` field's encoded bytes, present only until the next
/// decode attempt consumes them.
#[derive(Component, Debug, Clone)]
pub(crate) struct ImageBytes(pub Vec<u8>);

/// A prim's decoded image, present only once a decode has succeeded.
#[derive(Component)]
pub struct HsdImage(pub Handle<Image>);

/// `data` gates the whole image: without it there is nothing to decode, so
/// its removal tears down the image entirely.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), ParseError> {
    match name.field() {
        Some("sampler") => {
            let sampler = payload
                .map(hsd_image::ImageSampler::decode)
                .transpose()?
                .unwrap_or_default();
            commands.entity(prim).insert(HsdImageSampler(sampler));
        }
        Some("data") => match payload.map(hsd_image::ImageData::decode).transpose()? {
            Some(data) => {
                commands
                    .entity(prim)
                    .insert((ImageBytes(data.0), Pending::<HsdImage>::default()));
            }
            None => {
                commands
                    .entity(prim)
                    .remove::<(ImageBytes, HsdImage, Pending<HsdImage>)>();
            }
        },
        _ => {}
    }
    Ok(())
}

/// Decodes on every change to the encoded bytes, never on a sampler-only
/// change. The bytes are dropped afterwards, win or lose: only the decoded
/// asset is needed past this point.
pub(crate) fn rebuild_image(
    changed: Query<(Entity, &ImageBytes, Option<&HsdImageSampler>), Changed<ImageBytes>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for (prim, bytes, sampler) in &changed {
        let sampler = sampler.map_or_else(hsd_image::ImageSampler::default, |s| s.0);
        match decode(&bytes.0) {
            Ok(dyn_img) => {
                let handle = image_assets.add(build_img(dyn_img, sampler));
                commands.entity(prim).insert(HsdImage(handle));
            }
            Err(err) => {
                warn!("failed to decode image: {err}");
                commands.entity(prim).remove::<HsdImage>();
            }
        }
        commands
            .entity(prim)
            .remove::<(ImageBytes, Pending<HsdImage>)>();
    }
}

/// A sampler or `srgb` edit on an image already built mutates its asset in
/// place rather than re-decoding.
pub(crate) fn apply_sampler(
    changed: Query<(&HsdImage, &HsdImageSampler), Changed<HsdImageSampler>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    for (image, sampler) in &changed {
        let Some(mut asset) = image_assets.get_mut(&image.0) else {
            continue;
        };
        asset.sampler = ImageSampler::Descriptor(sampler_descriptor(sampler.0));
        asset.texture_descriptor.format = texture_format(sampler.0.srgb);
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

fn build_img(dyn_img: image::DynamicImage, sampler: hsd_image::ImageSampler) -> Image {
    let (width, height) = dyn_img.dimensions();
    let rgba = dyn_img.into_rgba8();

    let mut img = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        texture_format(sampler.srgb),
        RenderAssetUsages::default(),
    );

    img.sampler = ImageSampler::Descriptor(sampler_descriptor(sampler));

    img
}

fn sampler_descriptor(sampler: hsd_image::ImageSampler) -> ImageSamplerDescriptor {
    let mut descriptor = ImageSamplerDescriptor::default();
    for (value, target) in [
        (sampler.address_mode_u, &mut descriptor.address_mode_u),
        (sampler.address_mode_v, &mut descriptor.address_mode_v),
        (sampler.address_mode_w, &mut descriptor.address_mode_w),
    ] {
        if let Some(v) = value {
            *target = address_mode(v);
        }
    }
    for (value, target) in [
        (sampler.mag_filter, &mut descriptor.mag_filter),
        (sampler.min_filter, &mut descriptor.min_filter),
        (sampler.mipmap_filter, &mut descriptor.mipmap_filter),
    ] {
        if let Some(v) = value {
            *target = filter_mode(v);
        }
    }
    descriptor
}

fn texture_format(srgb: Option<bool>) -> TextureFormat {
    if srgb == Some(false) {
        TextureFormat::Rgba8Unorm
    } else {
        TextureFormat::Rgba8UnormSrgb
    }
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
