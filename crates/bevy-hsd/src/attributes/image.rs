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
    AttributeParser,
    ParseError,
};

const MAX_TEXTURE_DIMS: u32 = 8192;
/// Ceiling on what one decode may allocate, sized to hold the largest texture
/// the dimension cap admits.
const MAX_DECODE_BYTES: u64 = 4 * (MAX_TEXTURE_DIMS as u64) * (MAX_TEXTURE_DIMS as u64);

/// Assembled from `image/sampler` and `image/data`, each its own field so a
/// sampler change never re-decodes the image bytes.
#[derive(Component, Debug, Clone, Default)]
pub struct ImageData {
    pub sampler: hsd_image::ImageSampler,
    pub data:    Vec<u8>,
}

#[derive(Component, Default)]
pub struct HsdImage(pub Handle<Image>);

pub struct ImageParser;

impl AttributeParser for ImageParser {
    fn group(&self) -> &'static str {
        hsd_image::GROUP
    }

    /// `data` gates the whole image: without it there is nothing to decode,
    /// so its removal tears down the image entirely.
    fn lifecycle(
        &self,
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
                commands
                    .entity(prim)
                    .entry::<ImageData>()
                    .or_default()
                    .and_modify(move |mut data| data.sampler = sampler);
            }
            Some("data") => match payload.map(hsd_image::ImageData::decode).transpose()? {
                Some(data) => {
                    commands
                        .entity(prim)
                        .entry::<ImageData>()
                        .or_default()
                        .and_modify(move |mut image_data| image_data.data = data.0);
                    commands.entity(prim).insert(HsdImage::default());
                }
                None => {
                    commands.entity(prim).remove::<(ImageData, HsdImage)>();
                }
            },
            _ => {}
        }
        Ok(())
    }
}

pub fn rebuild_image(
    changed: Query<(Entity, &ImageData), Changed<ImageData>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for (prim, data) in &changed {
        let bytes = &data.data;
        if bytes.is_empty() {
            continue;
        }
        let sampler_attr = &data.sampler;
        let mut sampler = ImageSamplerDescriptor::default();
        for (value, target) in [
            (sampler_attr.address_mode_u, &mut sampler.address_mode_u),
            (sampler_attr.address_mode_v, &mut sampler.address_mode_v),
            (sampler_attr.address_mode_w, &mut sampler.address_mode_w),
        ] {
            if let Some(v) = value {
                *target = address_mode(v);
            }
        }
        for (value, target) in [
            (sampler_attr.mag_filter, &mut sampler.mag_filter),
            (sampler_attr.min_filter, &mut sampler.min_filter),
            (sampler_attr.mipmap_filter, &mut sampler.mipmap_filter),
        ] {
            if let Some(v) = value {
                *target = filter_mode(v);
            }
        }

        let dyn_img = match decode(bytes) {
            Ok(img) => img,
            Err(err) => {
                warn!("failed to decode image: {err}");
                commands.entity(prim).insert(HsdImage(Handle::default()));
                continue;
            }
        };

        let handle = image_assets.add(build_img(dyn_img, sampler, sampler_attr.srgb));
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
