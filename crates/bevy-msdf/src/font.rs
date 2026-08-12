//! A distance field grown at runtime: a closed budget of pages that generates
//! glyphs on demand, so any script a bundled face covers stays crisp without a
//! bake step per charset.

use std::{
    collections::HashSet,
    fmt::{
        Debug,
        Formatter,
    },
    sync::{
        Arc,
        Mutex,
        MutexGuard,
    },
};

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
use msdf::{
    atlas::LATIN,
    bake::DEFAULT_FONT,
    font::Font,
    generate::{
        Rendered,
        render,
    },
    runtime::{
        self,
        Atlas,
        Job,
        RuntimeOpts,
    },
};

/// The Latin charset is generated at startup, so the common case draws without
/// waiting on a generation frame.
const SEED: &str = LATIN;

/// A shared dynamic atlas and the GPU pages it has been copied into.
pub struct MsdfFont {
    state: Mutex<FontState>,
}

pub struct FontState {
    pub atlas:     runtime::Atlas,
    /// One image per page, in page order.
    pub pages:     Vec<Handle<Image>>,
    /// The distance range over one page. Uniform across pages because every
    /// page is the same size.
    pub unit_range: Vec2,
    /// Characters some live text is asking for; requested again whenever one
    /// of them was refused.
    want:           HashSet<char>,
    /// Characters pinned against eviction because text draws them.
    live:           HashSet<char>,
    /// Generated at construction and never released.
    seed:           HashSet<char>,
}

impl MsdfFont {
    /// Generates `SEED`, then creates an image per page so the GPU has the
    /// starting atlas before any text requests a glyph.
    pub fn new(font: Arc<Font>, opts: RuntimeOpts, images: &mut Assets<Image>) -> Self {
        let unit_range = Vec2::splat(opts.generate.range as f32 / opts.page_size as f32);
        let mut atlas = Atlas::new(font, opts);
        let seed = SEED.chars().collect::<HashSet<_>>();
        let seed_chars = seed.iter().copied().collect::<Vec<_>>();
        let _ = atlas.request(&seed_chars);
        while let Some(job) = atlas.next_job() {
            atlas.commit(job.ch, &render_glyph(&atlas, &job));
        }
        atlas.acquire(&seed_chars);

        let pages = (0..atlas.page_count())
            .map(|index| images.add(page_image(&atlas, index as u32)))
            .collect();

        Self {
            state: Mutex::new(FontState {
                atlas,
                pages,
                unit_range,
                want: seed.clone(),
                live: seed.clone(),
                seed,
            }),
        }
    }

    pub fn state(&self) -> MutexGuard<'_, FontState> {
        self.state.lock().expect("font")
    }

    /// Queues whatever the live texts lack, then pins residents and unpins the
    /// abandoned, so an eviction never takes a glyph a mesh is drawing.
    pub fn sync(&self, text_chars: &[char]) {
        let mut state = self.state();
        state.want = state.seed.iter().copied().chain(text_chars.iter().copied()).collect();
        let _ = state.atlas.request(text_chars);
        let pinned = state
            .want
            .iter()
            .filter(|ch| !state.live.contains(ch) && state.atlas.resident(**ch))
            .copied()
            .collect::<Vec<_>>();
        state.atlas.acquire(&pinned);
        state.live.extend(pinned);
        let released = state
            .live
            .iter()
            .filter(|ch| !state.want.contains(ch))
            .copied()
            .collect::<Vec<_>>();
        state.atlas.release(&released);
        for ch in &released {
            state.live.remove(ch);
        }
    }
}

impl Debug for MsdfFont {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let state = self.state();
        f.debug_struct("MsdfFont")
            .field("pages", &state.pages.len())
            .field("budget", &state.atlas.budget())
            .finish()
    }
}

/// What a [`crate::text::MsdfText`] draws with when it names no font.
#[derive(Resource, Debug, Clone)]
pub struct DefaultFont(pub Arc<MsdfFont>);

pub fn register_default_font(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let font = match Font::parse(Arc::<[u8]>::from(DEFAULT_FONT)) {
        Ok(font) => font,
        Err(err) => {
            error!("the bundled font failed to parse: {err}");
            return;
        }
    };
    let font = MsdfFont::new(Arc::new(font), RuntimeOpts::default(), &mut images);
    commands.insert_resource(DefaultFont(Arc::new(font)));
}

/// The pool-side half of a job: turns a queued character into a field.
#[must_use]
pub fn render_glyph(atlas: &runtime::Atlas, job: &Job) -> Rendered {
    atlas
        .font()
        .with_face(|face| render(face, job.id, job.ch, job.upem, atlas.generate_opts()))
        .expect("a face that parsed once parses again")
}

/// A page as a GPU image. `Rgba8Unorm`, never `Rgba8UnormSrgb`: the texels are
/// signed distances, and gamma-decoding them bends every edge the shader is
/// about to measure.
#[must_use]
pub fn page_image(atlas: &runtime::Atlas, index: u32) -> Image {
    let rgba = atlas.page_image(index as usize).expect("page");
    let mut image = Image::new(
        Extent3d {
            width:                 rgba.width(),
            height:                rgba.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.as_raw().clone(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    // Clamped, because a glyph at the edge of a page would otherwise sample
    // the opposite edge and grow a stray limb.
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..Default::default()
    });
    image
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bevy::asset::Assets;
    use bevy::image::Image;
    use msdf::{
        atlas::GlyphSource,
        layout::{
            Align,
            LayoutOpts,
            layout,
        },
    };

    use super::*;

    fn font() -> Arc<MsdfFont> {
        let font = Font::parse(Arc::<[u8]>::from(DEFAULT_FONT)).expect("parse");
        let mut images = Assets::<Image>::default();
        Arc::new(MsdfFont::new(Arc::new(font), RuntimeOpts::default(), &mut images))
    }

    #[test]
    fn the_seed_covers_the_latin_charset() {
        let font = font();
        let state = font.state();
        let missing = LATIN
            .chars()
            .filter(|ch| state.atlas.glyph(*ch).is_none())
            .collect::<String>();
        assert!(missing.is_empty(), "no glyph for {missing:?}");
    }

    /// Centring uses advance widths including side bearings, but the reader
    /// judges the ink, so the two must not drift apart.
    #[test]
    fn centred_text_looks_centred() {
        let font = font();
        let state = font.state();
        for text in ["Places", "Fruit", "Tools", "iiii", "WWWW", "A", "."] {
            let laid = layout(
                text,
                &state.atlas,
                &LayoutOpts {
                    size: 1.0,
                    align: Align::Center,
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
        drop(state);
    }

    #[test]
    fn the_dynamic_font_kerns() {
        let font = font();
        let state = font.state();
        assert!(
            state.atlas.kern('A', 'V') < 0.0,
            "a font that bakes no pair adjustments sets text loose"
        );
        drop(state);
    }

    #[test]
    fn a_glyph_the_seed_lacks_generates_on_demand() {
        let font = font();
        {
            let mut state = font.state();
            assert!(!state.atlas.resident('α'), "greek is not seeded");
            let _ = state.atlas.request(&['α']);
            while let Some(job) = state.atlas.next_job() {
                let rendered = render_glyph(&state.atlas, &job);
                state.atlas.commit(job.ch, &rendered);
            }
            assert!(state.atlas.resident('α'));
            let glyph = state.atlas.glyph('α').expect("glyph");
            assert!(!glyph.plane.is_empty());
            drop(state);
        }
    }
}
