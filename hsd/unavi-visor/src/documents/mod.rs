//! The Documents lens: corner brackets around each document near the viewer,
//! in its author's colour, and a tag over the one looked at.
//!
//! Documents are found by following reference sites from the current space,
//! and boxed from their prims' meshes and colliders. Boxes are cached in each
//! document's own frame and posed by its live offset every frame, so a moving
//! document is followed without measuring it again. Documents no reference
//! reaches, such as an object a script spawned, cannot be listed by a guest
//! and are not drawn.

mod bounds;
mod brackets;
mod catalog;
mod cube;
mod find;
mod geometry;
mod label;

use unavi_lens::DocumentId;
use wired_guest::math::{
    Transform,
    Vec3,
};

use self::{
    brackets::{
        Brackets,
        Drawn,
        Level,
    },
    catalog::{
        Catalog,
        Entry,
    },
    find::Kind,
    geometry::{
        Aabb,
        brackets as bracket_bars,
        enters,
        nearest,
        nearest_corner,
        world_corners,
    },
    label::{
        Facts,
        Label,
        describe,
    },
};
use crate::{
    slots,
    wired::{
        perception::perception::{
            meta,
            profile,
        },
        scene::document::{
            Document,
            script_document,
        },
    },
};

/// Metres from the viewer within which a document's nearest corner must lie
/// for it to be drawn.
const RANGE: f32 = 30.0;

/// The most documents drawn at once.
const MAX_DRAWN: usize = 32;

/// The fraction of an edge a bracket covers, and the most metres it covers.
const FRACTION: f32 = 0.15;
const CAP: f32 = 0.25;

/// Metres from the eye within which a document may become attended.
const ATTEND_REACH: f32 = 12.0;

/// Metres above a document's highest corner its tag stands.
const ABOVE: f32 = 0.15;

/// Fixed ticks between rereads of what the tag says, about half a second.
const FACTS_TICKS: u32 = 32;

pub struct Documents {
    visor:    Document,
    catalog:  Catalog,
    brackets: Brackets,
    label:    Label,
    /// The document each set of brackets is on.
    slots:    Vec<Option<DocumentId>>,
    attended: Option<DocumentId>,
    facts_in: u32,
    worn:     bool,
    warned:   bool,
}

impl Documents {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            visor:    script_document()?,
            catalog:  Catalog::default(),
            brackets: Brackets::new()?,
            label:    Label::new()?,
            slots:    vec![None; MAX_DRAWN],
            attended: None,
            facts_in: 0,
            worn:     false,
            warned:   false,
        })
    }

    /// Finds, boxes and chooses what to draw, while the lens is shown. Not
    /// shown, or with no eye, everything is put away.
    pub fn fixed_update(&mut self, shown: bool, eye: Option<&Transform>) {
        let Some(eye) = eye.filter(|_| shown) else {
            self.put_away();
            return;
        };
        if !self.worn {
            self.worn = true;
            self.catalog.rewalk();
        }
        self.catalog.fixed_update();

        let near = self.near(eye.translation);
        let ids: Vec<_> = near
            .iter()
            .map(|&index| self.catalog.entries()[index].id)
            .collect();
        let mut keys = self.slots.clone();
        slots::assign(&mut keys, &ids);
        self.slots = keys;
        if let Err(err) = self.brackets.grow(ids.len())
            && !self.warned
        {
            eprintln!("visor: document brackets: {err:?}");
            self.warned = true;
        }

        self.attended = self.attend(eye, &near);
        self.tell();
    }

    /// Poses every set of brackets and the tag against where the
    /// documents are now.
    pub fn update(&mut self) {
        if !self.worn {
            return;
        }
        for slot in 0..self.brackets.count().min(self.slots.len()) {
            let drawn = self.slots[slot].and_then(|id| self.draw(id));
            self.brackets.stage(slot, drawn.as_ref());
        }
        self.brackets.flush();

        let top = self
            .attended
            .and_then(|id| self.entry(id))
            .and_then(|entry| self.world_box(entry))
            .map(|(aabb, root)| top_of(&aabb, &root));
        match top {
            Some(top) => self.label.stand(top + Vec3::Y * ABOVE),
            None => self.label.hide(),
        }
    }

    fn put_away(&mut self) {
        if !self.worn {
            return;
        }
        self.worn = false;
        self.slots.fill(None);
        self.attended = None;
        self.label.hide();
        for slot in 0..self.brackets.count() {
            self.brackets.stage(slot, None);
        }
        self.brackets.flush();
        self.catalog.clear();
    }

    fn entry(&self, id: DocumentId) -> Option<&Entry> {
        self.catalog.entries().iter().find(|entry| entry.id == id)
    }

    /// A measured document's cached box and where its frame is now.
    fn world_box(&self, entry: &Entry) -> Option<(Aabb, Transform)> {
        let aabb = entry.bounds.filter(|aabb| !aabb.is_empty())?;
        let root = self.visor.offset_to(&entry.doc)?;
        Some((aabb, root))
    }

    /// Indices of the measured documents with a corner near the viewer,
    /// nearest first.
    fn near(&self, eye: Vec3) -> Vec<usize> {
        let candidates: Vec<_> = self
            .catalog
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let (aabb, root) = self.world_box(entry)?;
                Some((index, nearest_corner(&aabb, &root, eye)))
            })
            .collect();
        nearest(&candidates, RANGE, MAX_DRAWN)
    }

    /// The smallest document the viewer's gaze enters from outside. The space
    /// the viewer is in is never attended: its box is often flat ground under
    /// the eye, which a gaze enters from above, and its tag would stand
    /// over the space's middle, not over anything looked at.
    fn attend(&self, eye: &Transform, near: &[usize]) -> Option<DocumentId> {
        let gaze = eye.forward();
        let mut best: Option<(f32, DocumentId)> = None;
        for &index in near {
            let entry = &self.catalog.entries()[index];
            if entry.kind == Kind::Space {
                continue;
            }
            let Some((aabb, root)) = self.world_box(entry) else {
                continue;
            };
            if enters(&aabb, &root, eye.translation, gaze, ATTEND_REACH).is_none() {
                continue;
            }
            let volume = aabb.volume() * root.scale.x * root.scale.y * root.scale.z;
            if best.is_none_or(|(smallest, _)| volume < smallest) {
                best = Some((volume, entry.id));
            }
        }
        best.map(|(_, id)| id)
    }

    /// Gives the tag to the attended document and rereads what it says.
    fn tell(&mut self) {
        let Some(id) = self.attended else {
            self.label.hide();
            return;
        };
        let moved = self.label.key != Some(id);
        self.label.claim(id);
        if self.facts_in > 0 && !moved {
            self.facts_in -= 1;
            return;
        }
        self.facts_in = FACTS_TICKS;
        let Some(entry) = self.entry(id) else {
            return;
        };
        let name = meta(id).ok().flatten().and_then(|meta| meta.name);
        let author_name = entry
            .author
            .as_deref()
            .and_then(|did| profile(did).ok().flatten())
            .and_then(|profile| profile.name);
        let text = describe(&Facts {
            id,
            name: name.as_deref(),
            author_name: author_name.as_deref(),
            author: entry.author.as_deref(),
        });
        self.label.say(text);
    }

    fn draw(&self, id: DocumentId) -> Option<Drawn> {
        let entry = self.entry(id)?;
        let (aabb, root) = self.world_box(entry)?;
        let level = if self.attended == Some(id) {
            Level::Raised
        } else if entry.writable {
            Level::Solid
        } else {
            Level::Dim
        };
        Some(Drawn {
            bars: bracket_bars(&aabb, &root, FRACTION, CAP),
            rotation: root.rotation,
            color: entry.color,
            level,
        })
    }
}

/// The middle of the highest face of the posed box.
fn top_of(aabb: &Aabb, root: &Transform) -> Vec3 {
    let corners = world_corners(aabb, root);
    let sum: Vec3 = corners.iter().copied().sum();
    let high = corners
        .iter()
        .map(|corner| corner.y)
        .fold(f32::MIN, f32::max);
    let middle = sum / 8.0;
    Vec3::new(middle.x, high, middle.z)
}
