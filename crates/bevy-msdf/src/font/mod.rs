//! A distance field grown at runtime: a closed budget of pages that generates
//! glyphs on demand, so any script a loaded face covers stays crisp without a
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
    tasks::{
        ComputeTaskPool,
        TaskPool,
    },
};
use msdf::{
    atlas::{
        Glyph,
        GlyphSource,
        VerticalMetrics,
    },
    font::{
        Font,
        FontError,
    },
    generate::render,
    runtime::{
        self,
        Atlas,
        RuntimeOpts,
    },
};

use crate::material::unit_range;

pub mod asset;

/// Faces one stack may hold. Each one costs its own atlas pages, and every
/// character a text draws walks the stack, so a peer feeding fonts in cannot
/// be allowed to grow it without end.
pub const MAX_FONTS: usize = 8;

/// A shared dynamic atlas and the GPU pages it has been copied into.
pub struct MsdfFont {
    state: Mutex<FontState>,
}

pub struct FontState {
    pub atlas:      runtime::Atlas,
    /// One image per page, in page order.
    pub pages:      Vec<Handle<Image>>,
    /// The distance range over one page. Uniform across pages because every
    /// page is the same size.
    pub unit_range: Vec2,
    /// Characters pinned against eviction because text draws them.
    live:           HashSet<char>,
}

impl MsdfFont {
    /// Creates an image per page so the GPU has the atlas before any text
    /// requests a glyph. Every glyph generates on demand; nothing is
    /// pre-rendered.
    pub fn new(font: Arc<Font>, opts: RuntimeOpts, images: &mut Assets<Image>) -> Self {
        let atlas = Atlas::new(font, opts);
        let unit_range = unit_range(
            atlas.generate_opts().range as f32,
            atlas.budget().page_size,
        );

        let pages = (0..atlas.page_count())
            .map(|index| images.add(page_image(&atlas, index as u32)))
            .collect();

        Self {
            state: Mutex::new(FontState {
                atlas,
                pages,
                unit_range,
                live: HashSet::new(),
            }),
        }
    }

    pub fn state(&self) -> MutexGuard<'_, FontState> {
        self.state.lock().expect("font")
    }

    /// Queues whatever the live texts lack, then pins residents and unpins the
    /// abandoned, so an eviction never takes a glyph a mesh is drawing. A font
    /// no text draws is synced with nothing, which releases everything it had
    /// pinned.
    pub fn sync(&self, text_chars: &[char]) {
        let mut guard = self.state();
        let state = &mut *guard;
        let want = text_chars.iter().copied().collect::<HashSet<_>>();
        let _ = state.atlas.request(text_chars);

        let pinned = want
            .iter()
            .filter(|ch| !state.live.contains(ch) && state.atlas.resident(**ch))
            .copied()
            .collect::<Vec<_>>();
        state.atlas.acquire(&pinned);
        state.live.extend(pinned);

        let released = state
            .live
            .iter()
            .filter(|ch| !want.contains(ch))
            .copied()
            .collect::<Vec<_>>();
        state.atlas.release(&released);
        for ch in &released {
            state.live.remove(ch);
        }
        drop(guard);
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

/// Renders every job the atlas will hand out this round and commits the
/// results. Returns whether anything was generated, so a caller draining a
/// queue knows when to stop.
///
/// Generation is the expensive half of the pipeline — a field is sampled per
/// texel against every segment of the outline — so a round runs across the
/// compute pool rather than one glyph at a time on the caller's thread.
pub fn generate(atlas: &mut Atlas) -> bool {
    let mut jobs = Vec::new();
    while let Some(job) = atlas.next_job() {
        jobs.push(job);
    }
    if jobs.is_empty() {
        return false;
    }

    let rendered = {
        let face = atlas.font().face();
        let opts = atlas.generate_opts();
        ComputeTaskPool::get_or_init(TaskPool::default).scope(|scope| {
            for job in &jobs {
                scope.spawn(async move {
                    (job.ch, render(face, job.id, job.ch, job.upem, opts))
                });
            }
        })
    };
    for (ch, rendered) in rendered {
        atlas.commit(ch, &rendered);
    }
    true
}

/// What a [`crate::text::MsdfText`] draws with when it names no font: an
/// ordered fallback chain, primary first. Text resolves each character to the
/// first font in the stack that can render it.
#[derive(Resource, Debug, Clone, Default)]
pub struct DefaultFontStack(pub Vec<Arc<MsdfFont>>);

/// A fallback chain, used as the layout's [`GlyphSource`].
///
/// Each character resolves to the first font with a resident glyph for it,
/// then to the first face that could serve one, which advances the pen by the
/// width the landing glyph will have. A character no face covers draws the
/// primary font's `.notdef`.
pub struct FontStack {
    fonts: Vec<Arc<MsdfFont>>,
}

impl FontStack {
    #[must_use]
    pub const fn new(fonts: Vec<Arc<MsdfFont>>) -> Self {
        Self { fonts }
    }

    /// The first font whose face can serve `ch`, if any.
    #[must_use]
    pub fn serving(&self, ch: char) -> Option<usize> {
        self.fonts
            .iter()
            .position(|font| font.state().atlas.can_render(ch))
    }

    /// Whether any face in the chain covers `ch`. A character no face covers
    /// is one no amount of waiting will draw.
    #[must_use]
    pub fn can_render(&self, ch: char) -> bool {
        self.serving(ch).is_some()
    }

    #[must_use]
    pub fn font(&self, index: usize) -> Option<&Arc<MsdfFont>> {
        self.fonts.get(index)
    }

    fn stamped(&self, index: usize, ch: char) -> Option<Glyph> {
        let mut glyph = self.fonts.get(index)?.state().atlas.glyph(ch)?;
        glyph.font = index as u32;
        Some(glyph)
    }
}

impl GlyphSource for FontStack {
    fn vertical(&self) -> VerticalMetrics {
        self.fonts
            .first()
            .map(|font| font.state().atlas.vertical())
            .unwrap_or_default()
    }

    fn glyph(&self, ch: char) -> Option<Glyph> {
        let resident = self
            .fonts
            .iter()
            .position(|font| font.state().atlas.resident(ch));
        self.stamped(resident.or_else(|| self.serving(ch)).unwrap_or(0), ch)
    }

    fn kern(&self, left: char, right: char) -> f32 {
        let Some(index) = self.serving(left) else {
            return 0.0;
        };
        if self.serving(right) == Some(index) {
            self.fonts[index].state().atlas.kern(left, right)
        } else {
            0.0
        }
    }

    fn missing(&self, ch: char) -> bool {
        self.fonts
            .iter()
            .all(|font| !font.state().atlas.resident(ch))
    }
}

/// Builds a font from raw bytes, ready to be appended to a
/// [`DefaultFontStack`].
pub fn register_font(
    bytes: Arc<[u8]>,
    opts: RuntimeOpts,
    images: &mut Assets<Image>,
) -> Result<Arc<MsdfFont>, FontError> {
    let font = Font::parse(bytes)?;
    Ok(Arc::new(MsdfFont::new(Arc::new(font), opts, images)))
}

/// Raw font bytes a consumer fetched. Triggers appending the parsed font to
/// the [`DefaultFontStack`], so live text re-lays-out against it next frame.
///
/// The stack is append-only and order decides which face serves a character
/// several cover, so the first face registered is the primary.
#[derive(Event)]
pub struct RegisterFont(pub Arc<[u8]>);

pub(crate) fn on_register_font(
    trigger: On<RegisterFont>,
    mut stack: ResMut<DefaultFontStack>,
    mut images: ResMut<Assets<Image>>,
) {
    if stack.0.len() >= MAX_FONTS {
        error!("the fallback stack already holds {MAX_FONTS} fonts; dropping this one");
        return;
    }
    let bytes = Arc::clone(&trigger.event().0);
    match register_font(bytes, RuntimeOpts::default(), &mut images) {
        Ok(font) => stack.0.push(font),
        Err(err) => error!("failed to register font: {err}"),
    }
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

    use bevy::{
        asset::Assets,
        image::Image,
    };
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
        let font = Font::parse(Arc::<[u8]>::from(notosans::REGULAR_TTF)).expect("parse");
        let mut images = Assets::<Image>::default();
        Arc::new(MsdfFont::new(
            Arc::new(font),
            RuntimeOpts::default(),
            &mut images,
        ))
    }

    /// A font holding `text` resident, as one a frame of drawing has warmed.
    fn drawing(text: &str) -> Arc<MsdfFont> {
        let font = font();
        let chars = text.chars().collect::<Vec<_>>();
        font.sync(&chars);
        {
            let mut state = font.state();
            while generate(&mut state.atlas) {}
        }
        font.sync(&chars);
        font
    }

    #[test]
    fn a_new_font_holds_nothing_until_it_is_asked() {
        let font = font();
        let state = font.state();
        assert!(
            !state.atlas.resident('a'),
            "a face draws only what text asks it for"
        );
        assert!(state.atlas.can_render('a'), "and can serve that when asked");
        drop(state);
    }

    /// Centring uses advance widths including side bearings, but the reader
    /// judges the ink, so the two must not drift apart.
    #[test]
    fn centred_text_looks_centred() {
        let font = drawing("PlacesFruitTolsiWA.");
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
    fn a_requested_glyph_generates_on_demand() {
        let font = font();
        let mut state = font.state();
        let _ = state.atlas.request(&['α']);
        assert!(generate(&mut state.atlas));
        assert!(state.atlas.resident('α'));
        assert!(!state.atlas.glyph('α').expect("glyph").plane.is_empty());
        assert!(!generate(&mut state.atlas), "an empty queue generates nothing");
        drop(state);
    }

    #[test]
    fn register_font_parses_raw_bytes() {
        let mut images = Assets::<Image>::default();
        let font = register_font(
            Arc::<[u8]>::from(notosans::REGULAR_TTF),
            RuntimeOpts::default(),
            &mut images,
        )
        .expect("font");
        assert!(
            font.state().atlas.can_render('a'),
            "the registered face serves its glyphs"
        );
    }

    #[test]
    fn a_stack_resolves_characters_to_the_first_resident_font() {
        let stack = FontStack::new(vec![drawing("a"), drawing("a")]);

        let glyph = stack.glyph('a').expect("glyph");
        assert_eq!(glyph.font, 0, "both fonts drew 'a', the primary wins");
        assert!(!stack.missing('a'));
        assert!(
            stack.glyph('漢').is_some(),
            "the primary placeholder still draws"
        );
        assert!(stack.missing('漢'), "no face in the stack covers CJK");
        assert!(!stack.can_render('漢'));
    }

    #[test]
    fn a_character_only_a_fallback_has_drawn_is_measured_by_that_fallback() {
        let stack = FontStack::new(vec![font(), drawing("a")]);
        assert_eq!(stack.serving('a'), Some(0), "the primary face covers it");
        let glyph = stack.glyph('a').expect("glyph");
        assert_eq!(glyph.font, 1, "only the second font has drawn it");
        assert!(glyph.advance > 0.0);
    }

    #[test]
    fn an_empty_stack_measures_nothing_rather_than_panicking() {
        let stack = FontStack::new(Vec::new());
        assert!(stack.glyph('a').is_none());
        assert_eq!(stack.vertical(), VerticalMetrics::default());
        assert!(stack.kern('A', 'V').abs() < f32::EPSILON);
    }

    #[test]
    fn a_stack_does_not_kern_across_fonts() {
        let stack = FontStack::new(vec![font(), font()]);
        let direct = stack.fonts[0].state().atlas.kern('A', 'V');
        assert!(
            (stack.kern('A', 'V') - direct).abs() < 1.0e-6,
            "a pair in the same font kerns as that font kerns"
        );
    }

    #[test]
    fn registering_font_bytes_appends_to_the_default_stack() {
        let mut app = App::new();
        app.init_resource::<DefaultFontStack>()
            .init_resource::<Assets<Image>>()
            .add_observer(on_register_font);

        app.world_mut()
            .trigger(RegisterFont(Arc::<[u8]>::from(notosans::REGULAR_TTF)));

        let stack = app.world().resource::<DefaultFontStack>();
        assert_eq!(stack.0.len(), 1, "the parsed face joins the chain");
        assert!(stack.0[0].state().atlas.can_render('a'));
    }

    #[test]
    fn the_stack_stops_growing_at_its_cap() {
        let mut app = App::new();
        app.init_resource::<DefaultFontStack>()
            .init_resource::<Assets<Image>>()
            .add_observer(on_register_font);

        for _ in 0..MAX_FONTS + 2 {
            app.world_mut()
                .trigger(RegisterFont(Arc::<[u8]>::from(notosans::REGULAR_TTF)));
        }

        assert_eq!(app.world().resource::<DefaultFontStack>().0.len(), MAX_FONTS);
    }

    #[test]
    fn syncing_a_font_no_text_draws_releases_what_it_had_pinned() {
        let font = font();
        font.sync(&['a']);
        {
            let mut state = font.state();
            assert!(generate(&mut state.atlas));
            drop(state);
        }
        font.sync(&['a']);
        assert!(font.state().live.contains(&'a'), "a live character is pinned");

        font.sync(&[]);
        assert!(
            font.state().live.is_empty(),
            "a font nothing draws holds nothing against eviction"
        );
    }
}
