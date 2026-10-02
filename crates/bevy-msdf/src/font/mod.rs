//! A distance field grown at runtime: a closed budget of pages that generates
//! glyphs on demand, off the main thread.

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
    asset::{
        AssetId,
        RenderAssetUsages,
    },
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
        AsyncComputeTaskPool,
        Task,
        TaskPool,
        futures_lite::future::{
            block_on,
            poll_once,
        },
    },
};
use image::RgbaImage;
use msdf::{
    atlas::{
        Atlas,
        AtlasOpts,
        AtlasStats,
        DirtyRect,
    },
    field::{
        self,
        GlyphField,
    },
    font::{
        Font,
        FontError,
    },
    glyph::{
        Glyph,
        GlyphSource,
        VerticalMetrics,
    },
};

use crate::material::unit_range;

pub mod asset;

/// Faces one stack may hold. Each costs its own atlas pages, and every
/// character a text draws walks the stack.
pub const MAX_FONTS: usize = 8;

/// Glyph landings committed per [`MsdfFont::pump`] call. Generation itself
/// runs off-thread; this only bounds how many finished fields are blitted
/// into a page on the main thread in one frame.
const MAX_COMMITS_PER_PUMP: usize = 16;

pub(crate) struct FontState {
    atlas:      Atlas,
    /// One image per page, in page order.
    pages:      Vec<Handle<Image>>,
    /// The distance range over one page. Uniform across pages because every
    /// page is the same size.
    unit_range: Vec2,
    /// Characters pinned against eviction because text draws them.
    pinned:     HashSet<char>,
    /// Glyph generations handed to the compute pool, polled each
    /// [`MsdfFont::pump`] and committed once finished.
    jobs:       Vec<Task<(char, GlyphField)>>,
}

/// Recovers from a poisoned lock by logging and taking the guard anyway,
/// rather than panicking the caller: a prior panic already lost whatever
/// state it was mutating, and the atlas is a cache that tolerates a torn
/// update far better than a cascading panic across every text in the scene.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| {
        error!("a font's lock was poisoned by a prior panic; recovering");
        poisoned.into_inner()
    })
}

/// A shared dynamic atlas and the GPU pages it has been copied into.
pub struct MsdfFont {
    state: Mutex<FontState>,
}

impl MsdfFont {
    /// Creates an image per page so the GPU has the atlas before any text
    /// requests a glyph. Every glyph generates on demand; nothing is
    /// pre-rendered.
    pub fn new(font: Arc<Font>, opts: AtlasOpts, images: &mut Assets<Image>) -> Self {
        let atlas = Atlas::new(font, opts);
        let unit_range = unit_range(atlas.generate_opts().range as f32, atlas.stats().page_size);

        let pages = (0..atlas.page_count())
            .map(|index| images.add(page_image(&atlas, index as u32)))
            .collect();

        Self {
            state: Mutex::new(FontState {
                atlas,
                pages,
                unit_range,
                pinned: HashSet::new(),
                jobs: Vec::new(),
            }),
        }
    }

    pub(crate) fn state(&self) -> MutexGuard<'_, FontState> {
        lock(&self.state)
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

        let newly_pinned = want
            .iter()
            .filter(|ch| !state.pinned.contains(ch) && state.atlas.resident(**ch))
            .copied()
            .collect::<Vec<_>>();
        state.atlas.acquire(&newly_pinned);
        state.pinned.extend(newly_pinned);

        let released = state
            .pinned
            .iter()
            .filter(|ch| !want.contains(ch))
            .copied()
            .collect::<Vec<_>>();
        state.atlas.release(&released);
        for ch in &released {
            state.pinned.remove(ch);
        }
        drop(guard);
    }

    /// Advances the atlas's idle-eviction clock.
    pub fn tick(&self, dt: f32) {
        self.state().atlas.tick(dt);
    }

    /// Spawns generation for whatever the atlas is ready to hand out, polls
    /// jobs already in flight, and commits up to [`MAX_COMMITS_PER_PUMP`] of
    /// the ones that finished. Returns whether anything landed this call.
    ///
    /// Each job owns an `Arc<Font>` and runs on
    /// [`AsyncComputeTaskPool`], so a cold run of CJK text costs worker-thread
    /// time rather than a main-thread hitch. On wasm the pool is a
    /// single-threaded executor that still yields to the browser between
    /// polls.
    pub fn pump(&self) -> bool {
        let mut guard = self.state();
        let state = &mut *guard;

        while let Some(job) = state.atlas.next_job() {
            let opts = *state.atlas.generate_opts();
            let task = AsyncComputeTaskPool::get_or_init(TaskPool::default)
                .spawn(async move { (job.ch, field::generate(&job.font, job.id, &opts)) });
            state.jobs.push(task);
        }

        let mut landed = false;
        let mut commits = 0;
        let mut index = 0;
        while index < state.jobs.len() {
            if commits >= MAX_COMMITS_PER_PUMP {
                break;
            }
            match block_on(poll_once(&mut state.jobs[index])) {
                Some((ch, field)) => {
                    let task = state.jobs.remove(index);
                    task.detach();
                    state.atlas.commit(ch, &field);
                    landed = true;
                    commits += 1;
                }
                None => index += 1,
            }
        }
        drop(guard);
        landed
    }

    #[must_use]
    pub fn can_render(&self, ch: char) -> bool {
        self.state().atlas.can_render(ch)
    }

    #[must_use]
    pub fn resident(&self, ch: char) -> bool {
        self.state().atlas.resident(ch)
    }

    #[must_use]
    pub fn unit_range(&self) -> Vec2 {
        self.state().unit_range
    }

    #[must_use]
    pub fn page(&self, index: u32) -> Option<Handle<Image>> {
        self.state().pages.get(index as usize).cloned()
    }

    #[must_use]
    pub fn stats(&self) -> AtlasStats {
        self.state().atlas.stats()
    }

    /// Grows the page images to match the atlas, applies every dirty rect to
    /// them, and returns the sub-rects a render-world upload should copy.
    pub(crate) fn drain_dirty(&self, images: &mut Assets<Image>) -> Vec<PageUpload> {
        let mut guard = self.state();
        let state = &mut *guard;
        let dirty = state.atlas.take_dirty();
        if dirty.is_empty() {
            drop(guard);
            return Vec::new();
        }
        while state.pages.len() < state.atlas.page_count() {
            let index = state.pages.len() as u32;
            let image = page_image(&state.atlas, index);
            state.pages.push(images.add(image));
        }

        let mut uploads = Vec::with_capacity(dirty.len());
        for rect in dirty {
            let Some(handle) = state.pages.get(rect.page as usize) else {
                continue;
            };
            let Some(image) = images.get_mut_untracked(handle) else {
                continue;
            };
            let Some(page) = state.atlas.page_image(rect.page as usize) else {
                continue;
            };
            let Some(data) = rows(page, rect) else {
                continue;
            };
            if image.width() == page.width()
                && let Some(target) = image.data.as_mut()
            {
                blit_region(target, page.width(), &data, rect);
            }
            uploads.push(PageUpload {
                image: handle.id(),
                rect,
                data,
            });
        }
        drop(guard);
        uploads
    }
}

/// One page sub-rect ready for the render world to copy to the GPU.
pub(crate) struct PageUpload {
    pub image: AssetId<Image>,
    pub rect:  DirtyRect,
    pub data:  Vec<u8>,
}

/// The rect's texels, row by row. `None` when the rect is not inside the page,
/// which no atlas produces but no slice should trust either.
fn rows(page: &RgbaImage, rect: DirtyRect) -> Option<Vec<u8>> {
    if rect.x + rect.w > page.width() || rect.y + rect.h > page.height() {
        return None;
    }
    let row_bytes = page.width() as usize * 4;
    let width = rect.w as usize * 4;
    let mut data = Vec::with_capacity(width * rect.h as usize);
    for row in 0..rect.h {
        let start = (rect.y + row) as usize * row_bytes + rect.x as usize * 4;
        data.extend_from_slice(page.as_raw().get(start..start + width)?);
    }
    Some(data)
}

fn blit_region(target: &mut [u8], width: u32, data: &[u8], rect: DirtyRect) {
    let image_row = width as usize * 4;
    let rect_row = rect.w as usize * 4;
    for row in 0..rect.h as usize {
        let start = (rect.y as usize + row) * image_row + rect.x as usize * 4;
        let (Some(dst), Some(src)) = (
            target.get_mut(start..start + rect_row),
            data.get(row * rect_row..(row + 1) * rect_row),
        ) else {
            return;
        };
        dst.copy_from_slice(src);
    }
}

impl Debug for MsdfFont {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let state = self.state();
        f.debug_struct("MsdfFont")
            .field("pages", &state.pages.len())
            .field("stats", &state.atlas.stats())
            .finish()
    }
}

/// What a [`crate::text::MsdfText`] draws with when it names no font: an
/// ordered fallback chain, primary first. Text resolves each character to the
/// first font in the stack that can render it.
#[derive(Resource, Debug, Clone, Default)]
pub struct DefaultFontStack(pub Vec<Arc<MsdfFont>>);

/// A fallback chain, used as the layout's [`GlyphSource`].
///
/// Each character resolves to the first face that can serve it, and that
/// font alone decides what draws: its resident glyph if it has generated
/// one, otherwise a blank advance while it is pending, or `.notdef` if no
/// face covers the character at all. A different font having the character
/// resident (because some other text drew it) is never consulted — the pin
/// set and the draw set must name the same font or an eviction in the other
/// one could corrupt a live mesh.
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
        self.fonts.iter().position(|font| font.can_render(ch))
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
            .map_or_default(|font| font.state().atlas.vertical())
    }

    fn glyph(&self, ch: char) -> Option<Glyph> {
        self.stamped(self.serving(ch).unwrap_or(0), ch)
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
        self.serving(ch)
            .is_none_or(|index| !self.fonts[index].resident(ch))
    }
}

/// Builds a font from raw bytes, ready to be appended to a
/// [`DefaultFontStack`].
pub fn register_font(
    bytes: Arc<[u8]>,
    opts: AtlasOpts,
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
    match register_font(bytes, AtlasOpts::default(), &mut images) {
        Ok(font) => stack.0.push(font),
        Err(err) => error!("failed to register font: {err}"),
    }
}

/// A page as a GPU image. `Rgba8Unorm`, never `Rgba8UnormSrgb`: the texels are
/// signed distances, and gamma-decoding them bends every edge the shader is
/// about to measure.
#[must_use]
pub fn page_image(atlas: &Atlas, index: u32) -> Image {
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
        glyph::GlyphSource,
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
            AtlasOpts::default(),
            &mut images,
        ))
    }

    /// A font holding `text` resident, as one a frame of drawing has warmed.
    /// Runs the pump synchronously to completion: the test asserts on the
    /// landed state, not on timing.
    fn drawing(text: &str) -> Arc<MsdfFont> {
        let font = font();
        let chars = text.chars().collect::<Vec<_>>();
        font.sync(&chars);
        for _ in 0..200 {
            if !font.pump() && font.state().jobs.is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        font.sync(&chars);
        font
    }

    #[test]
    fn a_new_font_holds_nothing_until_it_is_asked() {
        let font = font();
        assert!(
            !font.resident('a'),
            "a face draws only what text asks it for"
        );
        assert!(font.can_render('a'), "and can serve that when asked");
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
            );
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
    fn a_requested_glyph_generates_on_demand() {
        let font = font();
        let _ = font.state().atlas.request(&['α']);
        for _ in 0..200 {
            if font.resident('α') {
                break;
            }
            font.pump();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(font.resident('α'));
        assert!(
            !font
                .state()
                .atlas
                .glyph('α')
                .expect("glyph")
                .plane
                .is_empty()
        );
    }

    #[test]
    fn a_stack_resolves_characters_to_the_first_font_that_can_serve_them() {
        let stack = FontStack::new(vec![drawing("a"), drawing("a")]);

        let glyph = stack.glyph('a').expect("glyph");
        assert_eq!(glyph.font, 0, "both fonts can serve 'a', the primary wins");
        assert!(!stack.missing('a'));
        assert!(
            stack.glyph('漢').is_some(),
            "the primary placeholder still draws"
        );
        assert!(stack.missing('漢'), "no face in the stack covers CJK");
        assert!(!stack.can_render('漢'));
    }

    /// The bug this guards: a glyph only a fallback font has drawn must not
    /// be borrowed to draw a character the primary font is the one serving.
    /// If it were, evicting the fallback's copy while the primary is still
    /// "missing" it would corrupt a live mesh, because the pin tracked by
    /// `sync_fonts` lives on the primary, not the fallback.
    #[test]
    fn a_character_the_primary_serves_is_measured_by_the_primary_even_if_a_fallback_drew_it() {
        let stack = FontStack::new(vec![font(), drawing("a")]);
        assert_eq!(stack.serving('a'), Some(0), "the primary face covers it");

        let glyph = stack.glyph('a').expect("glyph");
        assert_eq!(
            glyph.font, 0,
            "the serving font draws it, not a resident glyph borrowed from elsewhere in the stack"
        );
        assert!(
            glyph.plane.is_empty(),
            "the primary has not generated its own copy yet"
        );
        assert!(
            stack.missing('a'),
            "the font that will draw it has not drawn it yet"
        );
    }

    #[test]
    fn an_empty_stack_measures_nothing_rather_than_panicking() {
        let stack = FontStack::new(Vec::new());
        assert!(stack.glyph('a').is_none());
        assert_eq!(stack.vertical(), VerticalMetrics::default());
        assert!(stack.kern('A', 'V').abs() < f32::EPSILON);
    }

    #[test]
    fn a_stack_only_kerns_a_pair_the_same_font_serves_both_halves_of() {
        let stack = FontStack::new(vec![font()]);
        let direct = stack.fonts[0].state().atlas.kern('A', 'V');
        assert!(
            (stack.kern('A', 'V') - direct).abs() < 1.0e-6,
            "a pair the serving font covers kerns as that font kerns"
        );
        assert_eq!(
            stack.kern('A', '漢'),
            0.0,
            "no face in the stack serves the second character, so the pair does not kern"
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
        assert!(stack.0[0].can_render('a'));
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

        assert_eq!(
            app.world().resource::<DefaultFontStack>().0.len(),
            MAX_FONTS
        );
    }

    #[test]
    fn syncing_a_font_no_text_draws_releases_what_it_had_pinned() {
        let font = font();
        font.sync(&['a']);
        for _ in 0..200 {
            if font.resident('a') {
                break;
            }
            font.pump();
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        font.sync(&['a']);
        assert!(
            font.state().pinned.contains(&'a'),
            "a live character is pinned"
        );

        font.sync(&[]);
        assert!(
            font.state().pinned.is_empty(),
            "a font nothing draws holds nothing against eviction"
        );
    }

    /// Generation runs off-thread; the glyph is not resident on the same call
    /// that queued it, but repeated pumping eventually lands it.
    #[test]
    fn a_glyph_arrives_eventually_through_the_async_path() {
        let font = font();
        let _ = font.state().atlas.request(&['Q']);
        assert!(!font.resident('Q'), "generation has not run yet");

        let mut landed = false;
        for _ in 0..200 {
            if font.pump() && font.resident('Q') {
                landed = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(landed, "the glyph never arrived through the async path");
    }
}
