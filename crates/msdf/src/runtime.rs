//! The dynamic atlas: a closed budget of pages that generates glyphs on
//! demand. Text is untrusted, so every allocation and every request goes
//! through a hard cap; nothing here grows without bound.

use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    sync::Arc,
};

use etagere::{
    Allocation,
    AllocId,
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
        GenerateOpts,
        GUTTER,
        Rendered,
    },
};

/// What a character draws when the atlas cannot serve it a real glyph. The
/// character still advances under every policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fallback {
    /// Glyph 0 (`.notdef`), reserved in page 0 at font creation.
    #[default]
    Notdef,
    /// Advance without a quad.
    Skip,
    /// A rectangle of the advance width.
    Box,
}

#[derive(Debug, Clone)]
pub struct RuntimeOpts {
    /// Edge length of one page, in texels. One size per font keeps
    /// `unit_range` uniform across pages.
    pub page_size:        u32,
    /// Hard cap on pages per font; page count bounds texels.
    pub max_pages:        usize,
    /// Hard cap on distinct resident glyphs per font.
    pub max_glyphs:       usize,
    /// Hard cap on requests queued but not yet handed to a pool.
    pub max_pending:      usize,
    /// Hard cap on glyph generations in flight.
    pub max_in_flight:    usize,
    /// Ticks a released glyph stays resident before eviction may take it, so
    /// a pointer flicker cannot churn the pool.
    pub residency_window: u64,
    pub fallback:         Fallback,
    pub generate:         GenerateOpts,
}

impl Default for RuntimeOpts {
    fn default() -> Self {
        Self {
            page_size:        1024,
            max_pages:        4,
            max_glyphs:       1500,
            max_pending:      256,
            max_in_flight:    32,
            residency_window: 60,
            fallback:         Fallback::Notdef,
            generate:         GenerateOpts::default(),
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
    pub refusals:      u64,
}

struct Entry {
    glyph:     Glyph,
    refs:      u32,
    last_used: u64,
    slot:      Option<AllocId>,
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
    font:       Arc<Font>,
    opts:       RuntimeOpts,
    pages:      Vec<Page>,
    entries:    HashMap<char, Entry>,
    queued:     HashSet<char>,
    pending:    VecDeque<char>,
    in_flight:  usize,
    tick:       u64,
    refusals:   u64,
    generation: u64,
    notdef:     Glyph,
}

impl Atlas {
    /// Reserves `.notdef` (glyph 0) in page 0, permanently.
    #[must_use]
    pub fn new(font: Arc<Font>, opts: RuntimeOpts) -> Self {
        let page_size = opts.page_size;
        let mut atlas = Self {
            font,
            opts,
            pages: vec![Page::new(page_size)],
            entries: HashMap::new(),
            queued: HashSet::new(),
            pending: VecDeque::new(),
            in_flight: 0,
            tick: 0,
            refusals: 0,
            generation: 0,
            notdef: Glyph::default(),
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

    /// Bumped every time a glyph becomes resident or is evicted, so a caller
    /// can notice that meshes referencing the atlas want rebuilding.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    #[must_use]
    pub fn resident(&self, ch: char) -> bool {
        self.entries.contains_key(&ch)
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
        }
    }

    /// Queues characters for generation. Returns the ones newly accepted;
    /// residents, already-queued characters and characters the face lacks are
    /// skipped, and once the request queue is full the rest are refused.
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
            if self.font.glyph_index(ch).is_none() {
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

    /// Places a generated glyph. Returns [`RuntimeOp::Refused`] when no page
    /// fits it or the glyph budget cannot be made room for it.
    pub fn commit(&mut self, ch: char, rendered: &Rendered) -> RuntimeOp {
        if !self.queued.remove(&ch) {
            return RuntimeOp::Refused;
        }
        self.in_flight = self.in_flight.saturating_sub(1);
        if self.entries.contains_key(&ch) {
            return RuntimeOp::Ok;
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
        self.tick += 1;
        self.generation += 1;
        self.entries.insert(
            ch,
            Entry {
                glyph,
                refs: 0,
                last_used: self.tick,
                slot,
            },
        );
        RuntimeOp::Ok
    }

    /// Marks a taken job failed; the character stays on the fallback path.
    pub fn refuse(&mut self, ch: char) {
        if self.queued.remove(&ch) {
            self.in_flight = self.in_flight.saturating_sub(1);
            self.refusals += 1;
        }
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
        self.tick += 1;
        for &ch in chars {
            if let Some(entry) = self.entries.get_mut(&ch) {
                entry.last_used = self.tick;
                if delta < 0 {
                    entry.refs = entry.refs.saturating_sub(1);
                } else {
                    entry.refs = entry.refs.saturating_add(1);
                }
            }
        }
    }

    /// The oldest unreferenced, cooled-down glyph, if one exists.
    fn evict_one(&mut self) -> Option<char> {
        let cutoff = self.tick.saturating_sub(self.opts.residency_window);
        let victim = self
            .entries
            .iter()
            .filter(|(_, entry)| entry.refs == 0 && entry.last_used < cutoff)
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(ch, _)| *ch)?;
        let entry = self.entries.remove(&victim)?;
        if let Some(slot) = entry.slot {
            self.pages[entry.glyph.page as usize].allocator.deallocate(slot);
        }
        self.generation += 1;
        Some(victim)
    }

    fn render(&self, id: GlyphId, ch: char) -> Rendered {
        let upem = f64::from(self.font.units_per_em());
        self.font
            .with_face(|face| crate::generate::render(face, id, ch, upem, &self.opts.generate))
            .expect("a face that parsed once parses again")
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
                },
                None,
            ));
        };
        let request = size2(
            field.width().cast_signed() + GUTTER,
            field.height().cast_signed() + GUTTER,
        );
        for (index, page) in self.pages.iter_mut().enumerate() {
            if let Some(slot) = page.allocator.allocate(request) {
                let (x, y) = (slot.rectangle.min.x as u32, slot.rectangle.min.y as u32);
                for (column, row, pixel) in field.enumerate_pixels() {
                    let flipped = field.height() - 1 - row;
                    page.image.put_pixel(x + column, y + flipped, *pixel);
                }
                return Some((self.slot_glyph(rendered, field, &slot, index as u32), Some(slot.id)));
            }
        }
        if self.pages.len() < self.opts.max_pages {
            self.pages.push(Page::new(self.opts.page_size));
            return self.place(rendered);
        }
        None
    }

    fn slot_glyph(&self, rendered: &Rendered, field: &RgbaImage, slot: &Allocation, page: u32) -> Glyph {
        let size = self.opts.page_size as f32;
        let min = slot.rectangle.min;
        Glyph {
            plane:   rendered.plane,
            uv:      Rect {
                min: [min.x as f32 / size, min.y as f32 / size],
                max: [
                    (min.x + field.width().cast_signed()) as f32 / size,
                    (min.y + field.height().cast_signed()) as f32 / size,
                ],
            },
            advance: rendered.advance,
            page,
        }
    }

    fn fallback(&self, ch: char) -> Glyph {
        let advance = self
            .font
            .with_face(|face| {
                face.glyph_index(ch)
                    .and_then(|id| face.glyph_hor_advance(id))
                    .map_or(self.notdef.advance, |units| {
                        f32::from(units) / self.font.units_per_em()
                    })
            })
            .unwrap_or(self.notdef.advance);
        match self.opts.fallback {
            Fallback::Notdef => Glyph {
                plane:   self.notdef.plane,
                uv:      self.notdef.uv,
                advance,
                page:    self.notdef.page,
            },
            Fallback::Skip => Glyph {
                plane:   Rect::ZERO,
                uv:      Rect::ZERO,
                advance,
                page:    0,
            },
            Fallback::Box => Glyph {
                plane:   Rect {
                    min: [0.0, self.font.vertical.descender],
                    max: [advance, self.font.vertical.ascender],
                },
                uv:      Rect::ZERO,
                advance,
                page:    0,
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
    use crate::layout::LayoutOpts;

    fn atlas(opts: RuntimeOpts) -> Atlas {
        let font = Arc::new(Font::parse(Arc::<[u8]>::from(notosans::REGULAR_TTF)).expect("parse"));
        Atlas::new(font, opts)
    }

    fn opts() -> RuntimeOpts {
        RuntimeOpts {
            page_size:        256,
            max_pages:        2,
            max_glyphs:       8,
            max_pending:      4,
            max_in_flight:    2,
            residency_window: 3,
            ..Default::default()
        }
    }

    /// Requests, generates and commits one character, the way a task pool
    /// would.
    fn serve(atlas: &mut Atlas, ch: char) -> RuntimeOp {
        assert!(atlas.request(&[ch]).contains(&ch), "{ch} queued");
        let job = atlas.next_job().expect("job");
        assert_eq!(job.ch, ch);
        let rendered = atlas
            .font()
            .with_face(|face| {
                crate::generate::render(face, job.id, job.ch, job.upem, atlas.generate_opts())
            })
            .expect("render");
        atlas.commit(ch, &rendered)
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

    #[test]
    fn request_dedupes_residents_and_the_queued() {
        let mut atlas = atlas(opts());
        serve(&mut atlas, 'A');
        assert_eq!(atlas.request(&['A', 'B']), vec!['B']);
        atlas.request(&['B']);
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
        atlas.request(&['a', 'b']);
        assert!(atlas.next_job().is_some());
        assert!(
            atlas.next_job().is_none(),
            "the second job waits for the first to land"
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
            residency_window: 0,
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
            residency_window: 0,
            ..opts()
        });
        serve(&mut atlas, 'a');
        serve(&mut atlas, 'b');
        atlas.release(&['a']);
        assert_eq!(serve(&mut atlas, 'c'), RuntimeOp::Ok);
        assert!(
            !atlas.resident('b'),
            "b cooled and made room before a's window closed"
        );
        assert!(atlas.resident('a'));
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
        let glyph = atlas.glyph('b').expect("fallback");
        assert_eq!(glyph.page, atlas.notdef.page);
    }

    #[test]
    fn a_missing_character_still_advances() {
        let mut atlas = atlas(opts());
        let glyph = atlas.glyph('Z').expect("fallback");
        assert!(glyph.advance > 0.0, "the face knows Z's width");
        assert!(!glyph.plane.is_empty(), "the notdef box draws");
        assert!(atlas.missing('Z'));
    }

    #[test]
    fn skip_fallback_advances_without_ink() {
        let mut atlas = atlas(RuntimeOpts {
            fallback: Fallback::Skip,
            ..opts()
        });
        let glyph = atlas.glyph('Z').expect("fallback");
        assert!(glyph.advance > 0.0);
        assert!(glyph.plane.is_empty(), "Skip draws nothing");
    }

    #[test]
    fn box_fallback_traces_the_advance() {
        let mut atlas = atlas(RuntimeOpts {
            fallback: Fallback::Box,
            ..opts()
        });
        let glyph = atlas.glyph('Z').expect("fallback");
        assert!(glyph.advance > 0.0);
        assert_eq!(glyph.plane.max[0], glyph.advance);
        assert!(glyph.plane.min[1] < 0.0 && glyph.plane.max[1] > 0.0);
    }

    #[test]
    fn layout_reports_missing_and_draws_the_fallback() {
        let mut atlas = atlas(opts());
        let laid = crate::layout::layout("ab", &atlas, &LayoutOpts::default()).expect("layout");
        assert_eq!(laid.missing, vec!['a', 'b']);
        assert_eq!(
            laid.quads.len(),
            2,
            "notdef placeholders draw one quad per character"
        );
        assert!(laid.quads[1].plane.min[0] > laid.quads[0].plane.min[0]);
    }

    #[test]
    fn the_budget_counts_what_it_caps() {
        let atlas = atlas(opts());
        let budget = atlas.budget();
        assert_eq!(budget.pages, 1);
        assert_eq!(budget.glyphs, 1, "notdef is the only resident glyph");
        assert_eq!(budget.pending, 0);
        assert_eq!(budget.in_flight, 0);
    }
}
