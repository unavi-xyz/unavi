//! The documents in the space, with their authors and cached boxes.

use unavi_lens::DocumentId;
use wired_guest::{
    color::user_color,
    math::Color,
};

use super::{
    bounds::measure,
    find::{
        Found,
        Kind,
        find,
    },
    geometry::Aabb,
};
use crate::wired::{
    peer::authority::author,
    scene::document::Document,
};

/// Fixed ticks between walks for documents, about three seconds.
const DISCOVER_TICKS: u32 = 192;

/// Documents checked for a change each fixed tick.
const BUDGET: usize = 3;

/// Fixed ticks after which a box is measured again whatever the document's
/// prim count, about half a minute.
const STALE_TICKS: u64 = 2048;

/// The colour of a document no user has proven they author.
const UNAUTHORED: Color = Color::rgb(0.8, 0.8, 0.8);

pub struct Entry {
    pub id:       DocumentId,
    pub doc:      Document,
    pub kind:     Kind,
    /// The user holding the document's write key.
    pub author:   Option<String>,
    pub color:    Color,
    /// Whether the local user may write it.
    pub writable: bool,
    /// In the document's root frame, `None` until first measured.
    pub bounds:   Option<Aabb>,
    prims:        usize,
    measured_at:  u64,
}

impl Entry {
    fn new(found: Found) -> Self {
        let mut entry = Self {
            id:          found.id,
            doc:         found.doc,
            kind:        found.kind,
            author:      None,
            color:       UNAUTHORED,
            writable:    false,
            bounds:      None,
            prims:       0,
            measured_at: 0,
        };
        entry.read_authority();
        entry
    }

    fn read_authority(&mut self) {
        self.author = author(&self.doc).ok().flatten();
        self.color = self.author.as_deref().map_or(UNAUTHORED, user_color);
        self.writable = self.doc.may_write();
    }

    /// Measures again if the document gained or lost prims, was never
    /// measured, or was measured long ago.
    fn check(&mut self, now: u64) {
        let prims = self.doc.prims().len();
        let stale = now.saturating_sub(self.measured_at) > STALE_TICKS;
        if self.bounds.is_some() && prims == self.prims && !stale {
            return;
        }
        self.bounds = Some(measure(&self.doc));
        self.prims = prims;
        self.measured_at = now;
    }
}

#[derive(Default)]
pub struct Catalog {
    entries:     Vec<Entry>,
    cursor:      usize,
    now:         u64,
    discover_in: u32,
}

impl Catalog {
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Walks for documents when it is time, and checks a few for change.
    pub fn fixed_update(&mut self) {
        self.now += 1;
        if self.discover_in == 0 {
            self.discover_in = DISCOVER_TICKS;
            self.discover();
        }
        self.discover_in -= 1;

        let count = self.entries.len();
        for _ in 0..BUDGET.min(count) {
            self.cursor = (self.cursor + 1) % count;
            self.entries[self.cursor].check(self.now);
        }
    }

    /// Makes the next fixed update walk again.
    pub const fn rewalk(&mut self) {
        self.discover_in = 0;
    }

    /// Lets go of every document, such as when the lens is put away.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor = 0;
    }

    fn discover(&mut self) {
        let mut previous = std::mem::take(&mut self.entries);
        for found in find() {
            let entry = match previous.iter().position(|entry| entry.id == found.id) {
                Some(index) => {
                    let mut entry = previous.swap_remove(index);
                    entry.kind = found.kind;
                    entry.read_authority();
                    entry
                }
                None => Entry::new(found),
            };
            self.entries.push(entry);
        }
        self.cursor = 0;
    }
}
