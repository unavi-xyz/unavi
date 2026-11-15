//! One inspected prim: what the slate says about it, and the edits its rows
//! make. The edits are to the prim itself; how the element it belongs to is
//! placed is read from what it stands for as a whole (a referenced
//! document's site, a prim of the space, a loose document's root).

use std::collections::HashSet;

use wired_guest::math::{
    Transform,
    Vec3,
};

use self::edit::{
    Held,
    Nudge,
};
use crate::{
    element_of,
    unavi::vui::api::SlateEvent,
    wired::{
        core::ids::PrimRef,
        scene::{
            document::{
                Document,
                open_document,
            },
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

mod attributes;
mod edit;
mod format;
mod rows;

/// Most children the tree card lists.
const MAX_CHILDREN: usize = 8;

mod row {
    pub const NAME: u32 = 0;
    pub const DOCUMENT: u32 = 1;
    pub const LOCKED: u32 = 4;
    /// The position as a vector, then the turn and the scale.
    pub const POSITION: u32 = 10;
    pub const TURN: u32 = 13;
    pub const SCALE: u32 = 14;
    /// The tree card's heading, its parent and its children.
    pub const TREE: u32 = 30;
    pub const PARENT: u32 = 31;
    pub const CHILD: u32 = 32;
    pub const DELETE: u32 = 48;
    pub const MAKE_LOCAL: u32 = 49;
    pub const CLOSE: u32 = 50;
    /// Component cards count up from here, [`CARD_STRIDE`] ids each: the
    /// heading, then its fields.
    pub const CARD: u32 = 200;
    pub const CARD_STRIDE: u32 = 16;
}

/// The cards open on a new inspection: where it is.
const OPEN_AT_FIRST: [&str; 1] = [attributes::TRANSFORM];
/// The name the tree card is kept open by.
const TREE: &str = "Tree";

/// What a row press asked of the tool rather than of the element.
pub enum Asked {
    Nothing,
    Close,
    /// Inspect this prim instead: a parent or a child chosen from the rows.
    Inspect(PrimRef),
}

pub struct Inspection {
    /// The document holding the prim the hit stands for as a whole.
    placed_in: Document,
    loose:     bool,
    /// The reference site in `placed_in` placing the hit's document, when a
    /// reference does.
    site:      Option<(u64, u64)>,
    hit_doc:   Document,
    hit_prim:  (u64, u64),
    /// The point inspected, in the prim's own frame, which the slate's
    /// tether runs to as the prim moves.
    anchor:    Vec3,
    /// The cards shown open, by name.
    open:      HashSet<String>,
    /// The transform edit a scrub has open.
    held:      Option<Held>,
}

impl Inspection {
    /// Inspects what a hit on `target` at `point` stands for, or `None` when
    /// a document it needs is not loaded. Without a point, the tether runs to
    /// the prim's origin.
    #[must_use]
    pub fn of(target: PrimRef, point: Option<Vec3>) -> Option<Self> {
        let hit_doc = open_document(target.document).ok()??;
        let (placed_in, prim, loose) = element_of(target)?;
        let site = (!loose && placed_in.id() != hit_doc.id()).then_some(prim);
        let anchor = point
            .and_then(|point| {
                let pose = hit_doc.world_transform(target.prim).ok()?;
                Some(local_point(&pose, point))
            })
            .unwrap_or(Vec3::ZERO);
        Some(Self {
            placed_in,
            loose,
            site,
            hit_doc,
            hit_prim: target.prim,
            anchor,
            open: OPEN_AT_FIRST
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
            held: None,
        })
    }

    /// The prim inspected.
    #[must_use]
    pub fn target(&self) -> PrimRef {
        self.at(self.hit_prim)
    }

    /// Which cards are open, carried over to the next prim inspected from
    /// this slate.
    #[must_use]
    pub fn open_cards(&self) -> HashSet<String> {
        self.open.clone()
    }

    pub fn set_open_cards(&mut self, open: HashSet<String>) {
        self.open = open;
    }

    /// Folds the card named `name` if it is open, else opens it.
    fn flip(&mut self, name: &str) {
        if !self.open.remove(name) {
            self.open.insert(name.to_owned());
        }
    }

    /// Where the point inspected is now, in the script's frame.
    #[must_use]
    pub fn anchor(&self) -> Option<Vec3> {
        let pose = self.hit_doc.world_transform(self.hit_prim).ok()?;
        Some(pose.transform_point(self.anchor))
    }

    /// Answers what a row did.
    pub fn apply(&mut self, event: &SlateEvent) -> Asked {
        match *event {
            SlateEvent::Pressed(row::CLOSE) => return Asked::Close,
            SlateEvent::Pressed(row::DELETE) => {
                if let Err(err) = self.delete() {
                    eprintln!("inspector: delete: {err}");
                }
                return Asked::Close;
            }
            // The site opens the copy in place of what it showed, so this
            // inspection's document is gone.
            SlateEvent::Pressed(row::MAKE_LOCAL) => {
                match self.make_local() {
                    Ok(()) => println!("inspector: made local"),
                    Err(err) => eprintln!("inspector: make local: {err}"),
                }
                return Asked::Close;
            }
            SlateEvent::Pressed(row::PARENT) => {
                if let Some(parent) = self.parent_ref() {
                    return Asked::Inspect(parent);
                }
            }
            SlateEvent::Pressed(id)
                if (row::CHILD..row::CHILD + MAX_CHILDREN as u32).contains(&id) =>
            {
                if let Some(child) = self.child_ref((id - row::CHILD) as usize) {
                    return Asked::Inspect(child);
                }
            }
            SlateEvent::Pressed(row::TREE) => self.flip(TREE),
            SlateEvent::Pressed(id)
                if id >= row::CARD && (id - row::CARD).is_multiple_of(row::CARD_STRIDE) =>
            {
                let index = ((id - row::CARD) / row::CARD_STRIDE) as usize;
                if let Some(card) = self.cards().into_iter().nth(index) {
                    self.finish();
                    self.flip(&card.name);
                }
            }
            SlateEvent::Toggled((row::LOCKED, on)) => {
                if let Err(err) = self.lock(on) {
                    eprintln!("inspector: lock: {err}");
                }
            }
            SlateEvent::Scrubbed((id, part, by)) => {
                if let Some(nudge) = Nudge::of(id, part, by)
                    && let Err(err) = self.nudge(nudge)
                {
                    eprintln!("inspector: nudge: {err:?}");
                }
            }
            SlateEvent::Released(_) => self.finish(),
            SlateEvent::Pressed(_) | SlateEvent::Toggled(_) | SlateEvent::Picked(_) => {}
        }
        Asked::Nothing
    }

    /// Whether the transform rows may change this prim.
    fn editable(&self) -> bool {
        self.hit_doc.may_write()
            && !self.locked()
            && matches!(
                self.hit_doc.get(self.hit_prim, &PropertyKey::Transform),
                Some(Property::Transform(_))
            )
    }

    pub fn parent_ref(&self) -> Option<PrimRef> {
        self.parent().map(|parent| self.at(parent))
    }

    pub fn child_ref(&self, index: usize) -> Option<PrimRef> {
        self.hit_doc
            .children(self.hit_prim)
            .get(index)
            .map(|child| self.at(*child))
    }

    fn parent(&self) -> Option<(u64, u64)> {
        match self.hit_doc.get(self.hit_prim, &PropertyKey::Parent) {
            Some(Property::Parent(parent)) => parent,
            _ => None,
        }
    }

    /// `prim` of the inspected document, as a hit on it would name it.
    fn at(&self, prim: (u64, u64)) -> PrimRef {
        PrimRef {
            document: self.hit_doc.id(),
            prim,
        }
    }
}

/// `point` in the frame `pose` places, exactly for any scale: what
/// `pose.transform_point` takes back to `point`. An axis scaled to nothing
/// gives no position along it.
fn local_point(pose: &Transform, point: Vec3) -> Vec3 {
    let unturned = pose.rotation.inverse() * (point - pose.translation);
    let unscale = |along: f32, by: f32| {
        if by.abs() < f32::EPSILON {
            0.0
        } else {
            along / by
        }
    };
    Vec3::new(
        unscale(unturned.x, pose.scale.x),
        unscale(unturned.y, pose.scale.y),
        unscale(unturned.z, pose.scale.z),
    )
}

#[cfg(test)]
mod tests {
    use wired_guest::math::Quat;

    use super::*;

    #[test]
    fn a_point_on_a_stretched_turned_prim_comes_back_where_it_was() {
        let pose = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from(glam::Quat::from_rotation_y(0.8)),
            Vec3::new(3.0, 1.0, 0.5),
        );
        let point = Vec3::new(2.0, 2.5, 2.0);
        let back = pose.transform_point(local_point(&pose, point));
        assert!((back - point).length() < 1.0e-4, "{back}");
    }
}
