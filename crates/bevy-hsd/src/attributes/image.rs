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
    attributes::image::{
        self as hsd_image,
        AddressMode,
        FilterMode,
    },
    property::{
        Payload,
        name::PropName,
    },
};
use image::GenericImageView;

const MAX_TEXTURE_DIMS: u32 = 8192;
/// RGBA8 at the dimension cap.
const MAX_DECODE_BYTES: u64 = 4 * (MAX_TEXTURE_DIMS as u64) * (MAX_TEXTURE_DIMS as u64);

/// The `image/sampler` field.
#[derive(Component, Debug, Clone, Default)]
pub(crate) struct HsdImageSampler(pub hsd_image::ImageSampler);

/// The `image/data` field, removed by the next decode.
#[derive(Component, Debug, Clone)]
pub(crate) struct ImageBytes(pub Vec<u8>);

/// A prim's successfully decoded image.
#[derive(Component)]
pub struct HsdImage(pub Handle<Image>);

/// Removing `data` tears down the whole image.
pub(crate) fn apply(
    commands: &mut Commands,
    prim: Entity,
    name: &PropName,
    payload: Option<&[u8]>,
) -> Result<(), postcard::Error> {
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
                commands.entity(prim).insert(ImageBytes(data.0));
            }
            None => {
                commands.entity(prim).remove::<(ImageBytes, HsdImage)>();
            }
        },
        _ => {}
    }
    Ok(())
}

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
        commands.entity(prim).remove::<ImageBytes>();
    }
}

/// Sampler edits mutate the built asset in place.
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

/// The limits go to the decoder itself, since a decompression bomb allocates
/// before `dimensions()` could be checked.
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
