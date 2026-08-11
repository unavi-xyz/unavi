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
use msdf::atlas::Atlas;

const LATIN_FIELD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/latin.png"));
const LATIN_METRICS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/latin.bin"));

/// A distance field and the metrics to lay text out against it.
#[derive(Asset, TypePath, Debug, Clone)]
pub struct MsdfFont {
    pub atlas: Atlas,
    pub field: Handle<Image>,
}

/// What a [`crate::text::MsdfText`] draws with when it names no font.
#[derive(Resource, Debug, Clone)]
pub struct DefaultFont(pub Handle<MsdfFont>);

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("decode field: {0}")]
    Image(#[from] image::ImageError),
    #[error("decode metrics: {0}")]
    Metrics(#[from] msdf::atlas::DecodeError),
}

/// Turns a distance field and its metrics into a font asset.
///
/// The field is `Rgba8Unorm` and never `Rgba8UnormSrgb`: the texels are signed
/// distances, and gamma-decoding them bends every edge the shader is about to
/// measure.
pub fn load(field: &[u8], metrics: &[u8]) -> Result<(Atlas, Image), FontError> {
    let atlas = Atlas::decode(metrics)?;
    let decoded = image::load_from_memory_with_format(field, image::ImageFormat::Png)?.to_rgba8();

    let mut image = Image::new(
        Extent3d {
            width:                 decoded.width(),
            height:                decoded.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        decoded.into_raw(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    // Clamped, because a glyph at the edge of the field would otherwise
    // sample the opposite edge and grow a stray limb.
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..Default::default()
    });

    Ok((atlas, image))
}

pub fn register_default_font(
    mut commands: Commands,
    mut fonts: ResMut<Assets<MsdfFont>>,
    mut images: ResMut<Assets<Image>>,
) -> Result {
    let (atlas, image) = load(LATIN_FIELD, LATIN_METRICS)?;
    let font = fonts.add(MsdfFont {
        atlas,
        field: images.add(image),
    });
    commands.insert_resource(DefaultFont(font));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn atlas() -> Atlas {
        load(LATIN_FIELD, LATIN_METRICS).expect("shipped font").0
    }

    #[test]
    fn the_shipped_font_covers_the_latin_charset() {
        let atlas = atlas();
        let missing = msdf::atlas::LATIN
            .chars()
            .filter(|ch| atlas.glyph(*ch).is_none())
            .collect::<String>();
        assert!(missing.is_empty(), "no glyph for {missing:?}");
    }

    /// Centring works on advance widths, which include each glyph's side
    /// bearings. What a reader judges is the ink, so the two must not drift
    /// far apart — this measures the real font rather than trusting the
    /// arithmetic.
    #[test]
    fn centred_text_looks_centred() {
        let atlas = atlas();
        for text in ["Places", "Fruit", "Tools", "iiii", "WWWW", "A", "."] {
            let laid = msdf::layout::layout(
                text,
                &atlas,
                &msdf::layout::LayoutOpts {
                    size: 1.0,
                    align: msdf::layout::Align::Center,
                    ..Default::default()
                },
            )
            .expect("layout");
            let drift = f32::midpoint(laid.ink.min[0], laid.ink.max[0]);
            let width = laid.ink.max[0] - laid.ink.min[0];
            assert!(
                drift.abs() < width * 0.02,
                "{text:?} ink centre is {drift} off, {}% of its width",
                (drift / width * 100.0).abs()
            );
        }
    }

    #[test]
    fn the_shipped_font_kerns() {
        assert!(
            atlas().kern('A', 'V') < 0.0,
            "a font that bakes no pair adjustments sets text loose"
        );
    }

    #[test]
    fn the_shipped_field_matches_its_metrics() {
        let (atlas, image) = load(LATIN_FIELD, LATIN_METRICS).expect("shipped font");
        assert_eq!(image.width(), atlas.width);
        assert_eq!(image.height(), atlas.height);
        assert_eq!(
            image.texture_descriptor.format,
            TextureFormat::Rgba8Unorm,
            "distances are not colours"
        );
    }
}
