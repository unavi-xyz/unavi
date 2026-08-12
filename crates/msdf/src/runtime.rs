//! The dynamic atlas: a closed budget of pages that generates glyphs on
//! demand. Text is untrusted, so every allocation and every request goes
//! through a hard cap; nothing here grows without bound.
//!
//! Residency is a cache. A glyph a mesh draws is pinned; once nothing draws it
//! it cools for [`RuntimeOpts::residency`] seconds, after which pressure may
//! evict the least recently used, and [`RuntimeOpts::idle_timeout`] seconds
//! after that the sweep takes it whether or not anything else needs the room.

use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    sync::Arc,
};

use etagere::{
    AllocId,
    Allocation,
    AtlasAllocator,
    size2,
};
use image::RgbaImage;
use ttf_parser::GlyphId;

use crate::{
    atlas::{
        Glyph,
        GlyphSource,
        Rect,
        VerticalMetrics,
    },
    font::Font,
    generate::{
        GUTTER,
        GenerateOpts,
        Rendered,
    },
};

/// Dirty rects held before they collapse into whole pages. A consumer that
/// uploads every frame never reaches this; one that stalls pays a full page
/// copy instead of an unbounded list.
const MAX_DIRTY: usize = 256;

/// Characters remembered as ungeneratable. Small: a face with more broken
/// glyphs than this is one no fallback stack should be consulting.
const MAX_DENIED: usize = 1024;

/// How often the idle sweep walks the resident set, in seconds.
const SWEEP_INTERVAL: f64 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct RuntimeOpts {
    /// Edge length of one page, in texels. One size per font keeps
    /// `unit_range` uniform across pages.
    pub page_size:     u32,
    /// Hard cap on pages per font; page count bounds texels.
    pub max_pages:     usize,
    /// Hard cap on distinct resident glyphs per font.
    pub max_glyphs:    usize,
    /// Hard cap on requests queued but not yet handed to a pool.
    pub max_pending:   usize,
    /// Hard cap on glyph generations in flight.
    pub max_in_flight: usize,
    /// Seconds a released glyph stays resident before eviction may take it, so
    /// a pointer flicker cannot churn the pool.
    pub residency:     f32,
    /// Seconds an unreferenced glyph survives without use before the sweep
    /// takes it, whether or not anything is waiting for the room.
    pub idle_timeout:  f32,
    pub generate:      GenerateOpts,
}

impl Default for RuntimeOpts {
    fn default() -> Self {
        Self {
            page_size:     1024,
            max_pages:     4,
            max_glyphs:    1500,
            max_pending:   256,
            max_in_flight: 32,
            residency:     1.0,
            idle_timeout:  60.0,
            generate:      GenerateOpts::default(),
        }
    }
}

/// One glyph handed to a task pool; the pool renders it and hands the result
/// back to [`Atlas::commit`].
#[derive(Debug, Clone, Copy)]
pub struct Job {
    pub ch:   char,
    pub id:   GlyphId,
    pub upem: f64,
}

/// The outcome of committing a generated glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeOp {
    /// The glyph is resident; the fallback policy no longer applies to it.
    Ok,
    /// The glyph could not be served (budget or eviction); it stays on the
    /// fallback path.
    Refused,
}

/// A page region that changed since the last [`Atlas::take_dirty`], in
/// texels. An upload can copy just this rect instead of the whole page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRect {
    pub page: u32,
    pub x:    u32,
    pub y:    u32,
    pub w:    u32,
    pub h:    u32,
}

/// A read-only view of the caps and their current use, for the dev tools and
/// a future per-document quota.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    pub max_pages:     usize,
    pub page_size:     u32,
    pub max_glyphs:    usize,
    pub max_pending:   usize,
    pub max_in_flight: usize,
    pub pages:         usize,
    pub glyphs:        usize,
    pub pending:       usize,
    pub in_flight:     usize,
    /// Glyphs the budget turned away; a climbing count means the caps are too
    /// tight for what the world is drawing.
    pub refusals:      u64,
    /// Glyphs the face could not render within the generator's caps. Non-zero
    /// means a malformed or hostile face.
    pub denied:        u64,
}

struct Entry {
    glyph: Glyph,
    refs:  u32,
    /// When the glyph was last drawn, against the residency and idle windows.
    used:  f64,
    /// Draw order, which is what least-recently-used means when a frame
    /// commits and releases several glyphs at the same instant.
    seq:   u64,
    slot:  Option<AllocId>,
}

struct Page {
    allocator: AtlasAllocator,
    image:     RgbaImage,
}

impl Page {
    fn new(size: u32) -> Self {
        Self {
            allocator: AtlasAllocator::new(size2(size.cast_signed(), size.cast_signed())),
            image:     RgbaImage::new(size, size),
        }
    }
}

pub struct Atlas {
    font:      Arc<Font>,
    opts:      RuntimeOpts,
    pages:     Vec<Page>,
    entries:   HashMap<char, Entry>,
    queued:    HashSet<char>,
    pending:   VecDeque<char>,
    /// Characters this face has an outline for that the generator refused.
    /// Never requested again, and reported as unrenderable so a fallback stack
    /// moves on to the next face.
    denied:    HashSet<char>,
    in_flight: usize,
    clock:     f64,
    swept:     f64,
    seq:       u64,
    refusals:  u64,
    denials:   u64,
    notdef:    Glyph,
    dirty:     Vec<DirtyRect>,
}

impl Atlas {
    /// Reserves `.notdef` (glyph 0) in page 0, permanently.
    #[must_use]
    pub fn new(font: Arc<Font>, opts: RuntimeOpts) -> Self {
        let page_size = opts.page_size.max(1);
        let mut atlas = Self {
            font,
            opts: RuntimeOpts {
                page_size,
                max_pages: opts.max_pages.max(1),
                // A field the gutter pushes past a page is one no page could
                // ever take, so it is refused before it is generated.
                generate: GenerateOpts {
                    max_field: opts
                        .generate
                        .max_field
                        .min(page_size.saturating_sub(GUTTER.unsigned_abs())),
                    ..opts.generate
                },
                ..opts
            },
            pages: vec![Page::new(page_size)],
            entries: HashMap::new(),
            queued: HashSet::new(),
            pending: VecDeque::new(),
            denied: HashSet::new(),
            in_flight: 0,
            clock: 0.0,
            swept: 0.0,
            seq: 0,
            refusals: 0,
            denials: 0,
            notdef: Glyph::default(),
            dirty: Vec::new(),
        };
        let rendered = atlas.render(GlyphId(0), '\u{0}');
        if let Some((glyph, _)) = atlas.place(&rendered) {
            atlas.notdef = glyph;
        }
        atlas
    }

    #[must_use]
    pub fn font(&self) -> &Font {
        &self.font
    }

    #[must_use]
    pub const fn generate_opts(&self) -> &GenerateOpts {
        &self.opts.generate
    }

    #[must_use]
    pub const fn page_count(&self) -> usize {
        self.pages.len()
    }

    #[must_use]
    pub fn page_image(&self, index: usize) -> Option<&RgbaImage> {
        self.pages.get(index).map(|page| &page.image)
    }

    /// Advances the residency clock and, once a sweep interval has passed,
    /// drops glyphs nothing has drawn for [`RuntimeOpts::idle_timeout`]. A
    /// caller that never advances the clock keeps its cache forever.
    pub fn advance(&mut self, delta: f32) {
        self.clock += f64::from(delta.max(0.0));
        if self.clock - self.swept < SWEEP_INTERVAL {
            return;
        }
        self.swept = self.clock;
        let cutoff = self.clock - f64::from(self.opts.idle_timeout.max(0.0));
        let stale = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.refs == 0 && entry.used <= cutoff)
            .map(|(ch, _)| *ch)
            .collect::<Vec<_>>();
        for ch in stale {
            self.evict(ch);
        }
    }

    /// Page regions changed since the last call, for an incremental upload.
    #[must_use]
    pub fn take_dirty(&mut self) -> Vec<DirtyRect> {
        std::mem::take(&mut self.dirty)
    }

    #[must_use]
    pub fn resident(&self, ch: char) -> bool {
        self.entries.contains_key(&ch)
    }

    /// Whether this face can serve `ch` at all: it has an outline the
    /// generator has not already refused. A fallback stack uses this to decide
    /// which font should serve a character.
    #[must_use]
    pub fn can_render(&self, ch: char) -> bool {
        !self.denied.contains(&ch) && self.font.glyph_index(ch).is_some()
    }

    #[must_use]
    pub fn budget(&self) -> Budget {
        Budget {
            max_pages:     self.opts.max_pages,
            page_size:     self.opts.page_size,
            max_glyphs:    self.opts.max_glyphs,
            max_pending:   self.opts.max_pending,
            max_in_flight: self.opts.max_in_flight,
            pages:         self.pages.len(),
            glyphs:        self.entries.len() + 1,
            pending:       self.pending.len(),
            in_flight:     self.in_flight,
            refusals:      self.refusals,
            denied:        self.denials,
        }
    }

    /// Queues characters for generation. Returns the ones newly accepted;
    /// residents, already-queued characters and characters this face cannot
    /// serve are skipped, and once the request queue is full the rest are
    /// refused.
    #[must_use]
    pub fn request(&mut self, chars: &[char]) -> Vec<char> {
        let mut accepted = Vec::new();
        for &ch in chars {
            if self.entries.contains_key(&ch) || self.queued.contains(&ch) {
                continue;
            }
            if self.pending.len() + self.in_flight >= self.opts.max_pending {
                self.refusals += 1;
                continue;
            }
            if !self.can_render(ch) {
                continue;
            }
            self.queued.insert(ch);
            self.pending.push_back(ch);
            accepted.push(ch);
        }
        accepted
    }

    /// Hands one queued character to the pool, up to `max_in_flight`.
    #[must_use]
    pub fn next_job(&mut self) -> Option<Job> {
        loop {
            if self.in_flight >= self.opts.max_in_flight {
                return None;
            }
            let ch = self.pending.pop_front()?;
            let Some(id) = self.font.glyph_index(ch) else {
                self.queued.remove(&ch);
                continue;
            };
            self.in_flight += 1;
            return Some(Job {
                ch,
                id,
                upem: f64::from(self.font.units_per_em()),
            });
        }
    }

    /// Places a generated glyph. Returns [`RuntimeOp::Refused`] when the
    /// generator turned the outline down, when no page fits it, or when the
    /// glyph budget cannot be made room for it.
    pub fn commit(&mut self, ch: char, rendered: &Rendered) -> RuntimeOp {
        if !self.queued.remove(&ch) {
            return RuntimeOp::Refused;
        }
        self.in_flight = self.in_flight.saturating_sub(1);
        if self.entries.contains_key(&ch) {
            return RuntimeOp::Ok;
        }
        if rendered.refused {
            self.deny(ch);
            return RuntimeOp::Refused;
        }
        while self.entries.len() >= self.opts.max_glyphs {
            if self.evict_one().is_none() {
                self.refusals += 1;
                return RuntimeOp::Refused;
            }
        }
        let Some((glyph, slot)) = self.place(rendered) else {
            self.refusals += 1;
            return RuntimeOp::Refused;
        };
        self.seq += 1;
        self.entries.insert(
            ch,
            Entry {
                glyph,
                refs: 0,
                used: self.clock,
                seq: self.seq,
                slot,
            },
        );
        RuntimeOp::Ok
    }

    /// Marks characters as referenced by a live mesh; a referenced glyph is
    /// never evicted.
    pub fn acquire(&mut self, chars: &[char]) {
        self.touch(chars, 1);
    }

    /// Drops the mesh references; a released glyph becomes evictable once its
    /// residency window passes.
    pub fn release(&mut self, chars: &[char]) {
        self.touch(chars, -1);
    }

    fn touch(&mut self, chars: &[char], delta: i32) {
        for &ch in chars {
            self.seq += 1;
            if let Some(entry) = self.entries.get_mut(&ch) {
                entry.used = self.clock;
                entry.seq = self.seq;
                if delta < 0 {
                    entry.refs = entry.refs.saturating_sub(1);
                } else {
                    entry.refs = entry.refs.saturating_add(1);
                }
            }
        }
    }

    fn deny(&mut self, ch: char) {
        self.denials += 1;
        if self.denied.len() >= MAX_DENIED {
            self.denied.clear();
        }
        self.denied.insert(ch);
    }

    /// Evicts the least recently used glyph that is unreferenced and has
    /// cooled past the residency window.
    fn evict_one(&mut self) -> Option<char> {
        let cutoff = self.clock - f64::from(self.opts.residency.max(0.0));
        let victim = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.refs == 0 && entry.used <= cutoff)
            .min_by_key(|(_, entry)| entry.seq)
            .map(|(ch, _)| *ch)?;
        self.evict(victim);
        Some(victim)
    }

    fn evict(&mut self, ch: char) {
        let Some(entry) = self.entries.remove(&ch) else {
            return;
        };
        if let Some(slot) = entry.slot
            && let Some(page) = self.pages.get_mut(entry.glyph.page as usize)
        {
            page.allocator.deallocate(slot);
        }
    }

    fn render(&self, id: GlyphId, ch: char) -> Rendered {
        let upem = f64::from(self.font.units_per_em());
        crate::generate::render(self.font.face(), id, ch, upem, &self.opts.generate)
    }

    /// Fits a field into a page, adding one if every existing page is full.
    /// Fields are blitted upside down: font space is y-up and an image is
    /// y-down. Returns the placed glyph and its slot, or `None` when no page
    /// fits it.
    fn place(&mut self, rendered: &Rendered) -> Option<(Glyph, Option<AllocId>)> {
        let Some(field) = &rendered.field else {
            return Some((
                Glyph {
                    plane:   rendered.plane,
                    uv:      Rect::ZERO,
                    advance: rendered.advance,
                    page:    0,
                    font:    0,
                },
                None,
            ));
        };
        let request = size2(
            field.width().cast_signed() + GUTTER,
            field.height().cast_signed() + GUTTER,
        );
        for (index, page) in self.pages.iter_mut().enumerate() {
            let Some(slot) = page.allocator.allocate(request) else {
                continue;
            };
            let (x, y) = (slot.rectangle.min.x as u32, slot.rectangle.min.y as u32);
            let (w, h) = (request.width as u32, request.height as u32);
            // The slot may hold an evicted glyph's texels, and the gutter is
            // never written by the blit; clearing first keeps a neighbour's
            // ink out of this glyph's edge samples.
            for row in 0..h {
                for column in 0..w {
                    page.image.put_pixel(x + column, y + row, image::Rgba([0; 4]));
                }
            }
            for (column, row, pixel) in field.enumerate_pixels() {
                let flipped = field.height() - 1 - row;
                page.image.put_pixel(x + column, y + flipped, *pixel);
            }
            let dirty = DirtyRect {
                page: index as u32,
                x,
                y,
                w,
                h,
            };
            let glyph = self.slot_glyph(rendered, field, &slot, index as u32);
            self.push_dirty(dirty);
            return Some((glyph, Some(slot.id)));
        }
        if self.pages.len() < self.opts.max_pages {
            self.pages.push(Page::new(self.opts.page_size));
            return self.place(rendered);
        }
        None
    }

    /// Collapses the list to whole pages once it is long enough that copying
    /// them outright is cheaper than tracking every rect.
    fn push_dirty(&mut self, rect: DirtyRect) {
        self.dirty.push(rect);
        if self.dirty.len() <= MAX_DIRTY {
            return;
        }
        let mut pages = self.dirty.iter().map(|rect| rect.page).collect::<Vec<_>>();
        pages.sort_unstable();
        pages.dedup();
        let size = self.opts.page_size;
        self.dirty = pages
            .into_iter()
            .map(|page| DirtyRect {
                page,
                x: 0,
                y: 0,
                w: size,
                h: size,
            })
            .collect();
    }

    fn slot_glyph(
        &self,
        rendered: &Rendered,
        field: &RgbaImage,
        slot: &Allocation,
        page: u32,
    ) -> Glyph {
        let size = self.opts.page_size as f32;
        let min = slot.rectangle.min;
        Glyph {
            plane: rendered.plane,
            uv: Rect {
                min: [min.x as f32 / size, min.y as f32 / size],
                max: [
                    (min.x + field.width().cast_signed()) as f32 / size,
                    (min.y + field.height().cast_signed()) as f32 / size,
                ],
            },
            advance: rendered.advance,
            page,
            font: 0,
        }
    }

    /// What a character draws while it has no field of its own. One the face
    /// can serve is waiting on generation, so it advances without ink rather
    /// than flashing a placeholder a landing glyph replaces; one the face
    /// cannot serve draws `.notdef`, the conventional tofu.
    fn fallback(&self, ch: char) -> Glyph {
        match self.font.advance(ch) {
            Some(advance) if !self.denied.contains(&ch) => Glyph {
                plane: Rect::ZERO,
                uv: Rect::ZERO,
                advance,
                page: 0,
                font: 0,
            },
            advance => Glyph {
                advance: advance.unwrap_or(self.notdef.advance),
                ..self.notdef
            },
        }
    }
}

impl GlyphSource for Atlas {
    fn vertical(&self) -> VerticalMetrics {
        self.font.vertical
    }

    fn glyph(&self, ch: char) -> Option<Glyph> {
        Some(
            self.entries
                .get(&ch)
                .map_or_else(|| self.fallback(ch), |entry| entry.glyph),
        )
    }

    fn kern(&self, left: char, right: char) -> f32 {
        self.font.kern(left, right)
    }

    fn missing(&self, ch: char) -> bool {
        !self.entries.contains_key(&ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        generate::render,
        layout::LayoutOpts,
        outline::Limits,
    };

    fn atlas(opts: RuntimeOpts) -> Atlas {
        let font = Arc::new(Font::parse(Arc::<[u8]>::from(notosans::REGULAR_TTF)).expect("parse"));
        Atlas::new(font, opts)
    }

    fn opts() -> RuntimeOpts {
        RuntimeOpts {
            page_size: 256,
            max_pages: 2,
            max_glyphs: 8,
            max_pending: 4,
            max_in_flight: 2,
            residency: 1.0,
            ..Default::default()
        }
    }

    /// Requests, generates and commits one character, the way a task pool
    /// would.
    fn serve(atlas: &mut Atlas, ch: char) -> RuntimeOp {
        assert!(atlas.request(&[ch]).contains(&ch), "{ch} queued");
        let job = atlas.next_job().expect("job");
        assert_eq!(job.ch, ch);
        let rendered = render(
            atlas.font().face(),
            job.id,
            job.ch,
            job.upem,
            atlas.generate_opts(),
        );
        atlas.commit(ch, &rendered)
    }

    /// Runs the clock past the residency window without tripping the sweep.
    fn cool(atlas: &mut Atlas) {
        atlas.clock += f64::from(atlas.opts.residency) + 0.001;
    }

    #[test]
    fn a_requested_character_becomes_resident() {
        let mut atlas = atlas(opts());
        assert_eq!(serve(&mut atlas, 'A'), RuntimeOp::Ok);
        assert!(atlas.resident('A'));
        let glyph = atlas.glyph('A').expect("glyph");
        assert!(!glyph.plane.is_empty(), "A has ink");
        assert!(!atlas.missing('A'));
    }

    #[test]
    fn a_committed_glyph_is_blitted_into_its_page() {
        let mut atlas = atlas(opts());
        assert_eq!(serve(&mut atlas, 'A'), RuntimeOp::Ok);
        let glyph = atlas.glyph('A').expect("glyph");
        let image = atlas.page_image(glyph.page as usize).expect("page");
        let size = image.width();
        let [x0, y0] = glyph.uv.min;
        let [x1, y1] = glyph.uv.max;
        let mut ink = 0;
        for column in (x0 * size as f32) as u32..(x1 * size as f32) as u32 {
            for row in (y0 * size as f32) as u32..(y1 * size as f32) as u32 {
                let pixel = image.get_pixel(column, row);
                if pixel.0[0] != 0 || pixel.0[1] != 0 || pixel.0[2] != 0 {
                    ink += 1;
                }
            }
        }
        assert!(
            ink > 0,
            "the field texels reached page {} at {x0:.3},{y0:.3}",
            glyph.page
        );
    }

    /// The page is scribbled over first, standing in for the texels an evicted
    /// glyph leaves in the slot the next one is handed.
    #[test]
    fn a_reused_slot_keeps_none_of_the_evicted_glyph() {
        let mut atlas = atlas(opts());
        for pixel in atlas.pages[0].image.pixels_mut() {
            *pixel = image::Rgba([255; 4]);
        }
        let _ = atlas.take_dirty();
        assert_eq!(serve(&mut atlas, '.'), RuntimeOp::Ok);

        let glyph = atlas.glyph('.').expect("glyph");
        let rect = *atlas.take_dirty().first().expect("rect");
        let image = atlas.page_image(glyph.page as usize).expect("page");
        let size = image.width() as f32;
        let texel = |uv: [f32; 2]| {
            [
                (uv[0] * size).round() as u32,
                (uv[1] * size).round() as u32,
            ]
        };
        let ([x0, y0], [x1, y1]) = (texel(glyph.uv.min), texel(glyph.uv.max));
        for row in rect.y..rect.y + rect.h {
            for column in rect.x..rect.x + rect.w {
                if (x0..x1).contains(&column) && (y0..y1).contains(&row) {
                    continue;
                }
                assert_eq!(
                    image.get_pixel(column, row).0,
                    [0; 4],
                    "what was in the slot survives beside the new field at {column},{row}"
                );
            }
        }
    }

    #[test]
    fn commit_reports_the_changed_rect_for_upload() {
        let mut atlas = atlas(opts());
        let notdef = atlas.take_dirty();
        assert_eq!(notdef.len(), 1, "notdef lands at construction");
        assert_eq!(serve(&mut atlas, 'A'), RuntimeOp::Ok);
        let dirty = atlas.take_dirty();
        assert_eq!(dirty.len(), 1, "one glyph, one rect");
        let rect = dirty[0];
        assert!(
            rect.w >= 4 && rect.h >= 4,
            "the rect covers the field plus gutter"
        );
        assert!(
            atlas.take_dirty().is_empty(),
            "a second take finds nothing new"
        );
    }

    #[test]
    fn a_consumer_that_never_uploads_does_not_grow_the_rect_list() {
        let mut atlas = atlas(RuntimeOpts {
            max_glyphs: 1,
            residency: 0.0,
            page_size: 64,
            generate: GenerateOpts {
                px_per_em: 8,
                ..Default::default()
            },
            ..opts()
        });
        let commits = MAX_DIRTY + 8;
        for index in 0..commits {
            atlas.clock += 1.0;
            let ch = if index % 2 == 0 { 'a' } else { 'b' };
            assert_eq!(serve(&mut atlas, ch), RuntimeOp::Ok, "{ch} lands");
        }
        let dirty = atlas.take_dirty();
        assert!(
            dirty.len() < commits,
            "{commits} commits left {} rects to replay",
            dirty.len()
        );
        assert!(
            dirty.iter().any(|rect| rect.w == 64 && rect.h == 64),
            "the list collapsed to whole pages"
        );
    }

    #[test]
    fn request_dedupes_residents_and_the_queued() {
        let mut atlas = atlas(opts());
        serve(&mut atlas, 'A');
        assert_eq!(atlas.request(&['A', 'B']), vec!['B']);
        let _ = atlas.request(&['B']);
        assert!(
            atlas.request(&['B']).is_empty(),
            "a queued character is not queued twice"
        );
    }

    #[test]
    fn a_character_the_face_lacks_is_never_queued() {
        let mut atlas = atlas(opts());
        assert!(atlas.request(&['漢']).is_empty());
        assert_eq!(atlas.budget().pending, 0);
    }

    #[test]
    fn the_request_queue_respects_max_pending() {
        let mut atlas = atlas(RuntimeOpts {
            max_pending: 2,
            ..opts()
        });
        assert_eq!(atlas.request(&['a', 'b', 'c', 'd']), vec!['a', 'b']);
        assert_eq!(atlas.budget().refusals, 2, "c and d were refused");
    }

    #[test]
    fn in_flight_caps_generation() {
        let mut atlas = atlas(RuntimeOpts {
            max_in_flight: 1,
            ..opts()
        });
        let _ = atlas.request(&['a', 'b']);
        assert!(atlas.next_job().is_some());
        assert!(
            atlas.next_job().is_none(),
            "the second job waits for the first to land"
        );
    }

    #[test]
    fn a_glyph_the_generator_refuses_is_never_asked_for_twice() {
        let mut atlas = atlas(RuntimeOpts {
            generate: GenerateOpts {
                outline: Limits { segments: 1 },
                ..Default::default()
            },
            ..opts()
        });
        assert_eq!(serve(&mut atlas, 'A'), RuntimeOp::Refused);
        assert_eq!(atlas.budget().denied, 1);
        assert!(
            !atlas.can_render('A'),
            "a stack falls through to the next face"
        );
        assert!(atlas.request(&['A']).is_empty());
        assert!(
            atlas.glyph('A').expect("fallback").advance > 0.0,
            "and the line still leaves room for it"
        );
    }

    #[test]
    fn a_full_page_overflows_into_the_next() {
        let mut atlas = atlas(RuntimeOpts {
            page_size: 64,
            max_pages: 3,
            max_glyphs: 100,
            generate: GenerateOpts {
                px_per_em: 16,
                ..Default::default()
            },
            ..opts()
        });
        for ch in "abcdefghij".chars() {
            assert_eq!(serve(&mut atlas, ch), RuntimeOp::Ok, "{ch} lands");
        }
        assert!(
            atlas.page_count() > 1,
            "ten glyphs do not fit a single 64px page"
        );
        for ch in "abcdefghij".chars() {
            let glyph = atlas.glyph(ch).expect("glyph");
            assert!(glyph.page < atlas.page_count() as u32);
            for corner in [glyph.uv.min, glyph.uv.max] {
                assert!(
                    (0.0..=1.0).contains(&corner[0]) && (0.0..=1.0).contains(&corner[1]),
                    "{ch} on page {} at {corner:?}",
                    glyph.page
                );
            }
        }
    }

    #[test]
    fn a_referenced_glyph_survives_pressure() {
        let mut atlas = atlas(RuntimeOpts {
            max_glyphs: 3,
            residency: 0.0,
            ..opts()
        });
        serve(&mut atlas, 'a');
        atlas.acquire(&['a']);
        serve(&mut atlas, 'b');
        serve(&mut atlas, 'c');
        assert_eq!(serve(&mut atlas, 'd'), RuntimeOp::Ok);
        assert!(atlas.resident('a'), "referenced glyphs are never evicted");
        assert!(!atlas.resident('b'), "b is the oldest unreferenced glyph");
        assert!(atlas.resident('d'));
    }

    #[test]
    fn the_residency_window_blocks_eviction() {
        let mut atlas = atlas(RuntimeOpts {
            max_glyphs: 2,
            ..opts()
        });
        serve(&mut atlas, 'a');
        serve(&mut atlas, 'b');
        assert_eq!(
            serve(&mut atlas, 'c'),
            RuntimeOp::Refused,
            "nothing is cold enough to evict, so the budget refuses"
        );
        assert!(atlas.resident('a'));
        assert!(atlas.resident('b'));
        assert_eq!(atlas.budget().refusals, 1);
    }

    #[test]
    fn a_released_glyph_evicts_once_cool() {
        let mut atlas = atlas(RuntimeOpts {
            max_glyphs: 2,
            ..opts()
        });
        serve(&mut atlas, 'a');
        atlas.acquire(&['a']);
        serve(&mut atlas, 'b');
        cool(&mut atlas);
        assert_eq!(serve(&mut atlas, 'c'), RuntimeOp::Ok);
        assert!(!atlas.resident('b'), "b cooled and made room");
        assert!(atlas.resident('a'), "a is still drawn");
    }

    #[test]
    fn an_idle_glyph_is_swept_without_anything_asking_for_the_room() {
        let mut atlas = atlas(RuntimeOpts {
            idle_timeout: 5.0,
            ..opts()
        });
        serve(&mut atlas, 'a');
        serve(&mut atlas, 'b');
        atlas.acquire(&['b']);
        atlas.advance(2.0);
        assert!(atlas.resident('a'), "still inside the idle window");
        atlas.advance(4.0);
        assert!(!atlas.resident('a'), "nothing drew a for five seconds");
        assert!(atlas.resident('b'), "b is still drawn");
        assert_eq!(atlas.budget().glyphs, 2, "the slot came back");
    }

    #[test]
    fn the_sweep_runs_at_most_once_an_interval() {
        let mut atlas = atlas(RuntimeOpts {
            idle_timeout: 0.0,
            ..opts()
        });
        serve(&mut atlas, 'a');
        atlas.advance(0.001);
        assert!(atlas.resident('a'), "a sweep per frame would be a scan per frame");
        atlas.advance(SWEEP_INTERVAL as f32);
        assert!(!atlas.resident('a'));
    }

    #[test]
    fn notdef_is_reserved_forever() {
        let mut atlas = atlas(RuntimeOpts {
            max_glyphs: 1,
            ..opts()
        });
        assert_eq!(serve(&mut atlas, 'a'), RuntimeOp::Ok);
        assert_eq!(
            serve(&mut atlas, 'b'),
            RuntimeOp::Refused,
            "a is warm, so nothing evicts to admit b"
        );
        assert!(atlas.resident('a'));
        assert_eq!(atlas.glyph('b').expect("fallback").page, atlas.notdef.page);
    }

    #[test]
    fn a_character_the_face_lacks_draws_tofu_and_still_advances() {
        let atlas = atlas(opts());
        let glyph = atlas.glyph('漢').expect("fallback");
        assert!(glyph.advance > 0.0, "notdef's own width carries the line");
        assert!(!glyph.plane.is_empty(), "the notdef box draws");
        assert!(atlas.missing('漢'));
    }

    #[test]
    fn a_character_waiting_on_generation_advances_without_flashing_tofu() {
        let mut atlas = atlas(opts());
        let _ = atlas.request(&['Z']);
        let glyph = atlas.glyph('Z').expect("fallback");
        assert!(glyph.advance > 0.0, "the face knows Z's width");
        assert!(glyph.plane.is_empty(), "and nothing is drawn in the meantime");
    }

    #[test]
    fn layout_reports_missing_and_leaves_room_for_it() {
        let atlas = atlas(opts());
        let laid = crate::layout::layout("ab", &atlas, &LayoutOpts::default()).expect("layout");
        assert_eq!(laid.missing, vec!['a', 'b']);
        assert!(laid.bounds.max[0] > 0.0, "the line is as wide as it will be");
    }

    #[test]
    fn the_budget_counts_what_it_caps() {
        let atlas = atlas(opts());
        let budget = atlas.budget();
        assert_eq!(budget.pages, 1);
        assert_eq!(budget.glyphs, 1, "notdef is the only resident glyph");
        assert_eq!(budget.pending, 0);
        assert_eq!(budget.in_flight, 0);
        assert_eq!(budget.denied, 0);
    }
}
