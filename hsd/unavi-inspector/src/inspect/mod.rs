//! One inspected prim: what the pane says about it, and the edits its
//! controls make. The edits are to the prim itself; how the element it belongs
//! to is placed is read from what it stands for as a whole (a referenced
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
    unavi::vui::api::PaneEvent,
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
mod blocks;
mod edit;
mod format;
mod hierarchy;
mod layers;

/// The ids a pane's controls and rows report by.
mod row {
    pub const LOCKED: u32 = 4;
    /// The position as three numbers, then the turn and the scale.
    pub const POSITION: u32 = 10;
    pub const POSITION_Z: u32 = 12;
    pub const TURN: u32 = 13;
    pub const SCALE: u32 = 14;
    pub const DELETE: u32 = 48;
    pub const MAKE_LOCAL: u32 = 49;
    pub const CLOSE: u32 = 50;
    /// Component cards count up from here, [`CARD_STRIDE`] ids each: the
    /// heading, then its fields.
    pub const CARD: u32 = 200;
    pub const CARD_STRIDE: u32 = 16;
    /// Prim hierarchy nodes count up from here, by their place in the list.
    pub const NODE: u32 = 1000;
    /// Layers count up from here.
    pub const LAYER: u32 = 2000;
}

/// The cards open on a new inspection: where it is.
const OPEN_AT_FIRST: [&str; 1] = [attributes::TRANSFORM];

/// What a press asked of the tool rather than of the element.
pub enum Asked {
    Nothing,
    Close,
    /// Inspect this prim instead: one chosen in the hierarchy.
    Inspect(PrimRef),
}

/// What an inspection has unfolded, carried to the next prim inspected from
/// the same pane.
pub struct Folds {
    cards: HashSet<String>,
    prims: HashSet<(u64, u64)>,
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
    /// The point inspected, in the prim's own frame, which the pane's tether
    /// runs to as the prim moves.
    anchor:    Vec3,
    /// The cards shown open, by name.
    open:      HashSet<String>,
    /// The prims of the hierarchy shown unfolded.
    unfolded:  HashSet<(u64, u64)>,
    /// The prim each hierarchy node stands for, as last built.
    nodes:     Vec<(u64, u64)>,
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
        let mut inspection = Self {
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
            unfolded: HashSet::new(),
            nodes: Vec::new(),
            held: None,
        };
        inspection.unfold_to_here();
        Some(inspection)
    }

    /// The prim inspected.
    #[must_use]
    pub fn target(&self) -> PrimRef {
        self.at(self.hit_prim)
    }

    /// What is unfolded, to carry to the next prim inspected from this pane.
    #[must_use]
    pub fn folds(&self) -> Folds {
        Folds {
            cards: self.open.clone(),
            prims: self.unfolded.clone(),
        }
    }

    pub fn set_folds(&mut self, folds: Folds) {
        self.open = folds.cards;
        self.unfolded.extend(folds.prims);
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

    /// Answers what a control did.
    pub fn apply(&mut self, event: &PaneEvent) -> Asked {
        match *event {
            PaneEvent::Pressed(row::CLOSE) => return Asked::Close,
            PaneEvent::Pressed(row::DELETE) => {
                if let Err(err) = self.delete() {
                    eprintln!("inspector: delete: {err}");
                }
                return Asked::Close;
            }
            // The site opens the copy in place of what it showed, so this
            // inspection's document is gone.
            PaneEvent::Pressed(row::MAKE_LOCAL) => {
                match self.make_local() {
                    Ok(()) => println!("inspector: made local"),
                    Err(err) => eprintln!("inspector: make local: {err}"),
                }
                return Asked::Close;
            }
            PaneEvent::Selected(id) | PaneEvent::Folded((id, _)) if is_card(id) => {
                let index = ((id - row::CARD) / row::CARD_STRIDE) as usize;
                if let Some(card) = self.cards().into_iter().nth(index) {
                    self.finish();
                    self.flip(&card.name);
                }
            }
            PaneEvent::Selected(id) => {
                if let Some(prim) = self.node(id).filter(|prim| *prim != self.hit_prim) {
                    return Asked::Inspect(self.at(prim));
                }
            }
            PaneEvent::Folded((id, open)) => {
                if let Some(prim) = self.node(id) {
                    if open {
                        self.unfolded.insert(prim);
                    } else {
                        self.unfolded.remove(&prim);
                    }
                }
            }
            PaneEvent::Toggled((row::LOCKED, on)) => {
                if let Err(err) = self.lock(on) {
                    eprintln!("inspector: lock: {err}");
                }
            }
            PaneEvent::Scrubbed((id, by)) => {
                if let Some(nudge) = Nudge::of(id, by)
                    && let Err(err) = self.nudge(nudge)
                {
                    eprintln!("inspector: nudge: {err:?}");
                }
            }
            PaneEvent::Released(_) => self.finish(),
            PaneEvent::Pressed(_)
            | PaneEvent::Toggled(_)
            | PaneEvent::Slid(_)
            | PaneEvent::Chosen(_)
            | PaneEvent::Followed(_) => {}
        }
        Asked::Nothing
    }

    /// The prim hierarchy node `id` stands for.
    fn node(&self, id: u32) -> Option<(u64, u64)> {
        let index = id.checked_sub(row::NODE)?;
        self.nodes.get(index as usize).copied()
    }

    /// Whether the transform controls may change this prim.
    fn editable(&self) -> bool {
        self.hit_doc.may_write()
            && !self.locked()
            && matches!(
                self.hit_doc.get(self.hit_prim, &PropertyKey::Transform),
                Some(Property::Transform(_))
            )
    }

    fn parent_of(&self, prim: (u64, u64)) -> Option<(u64, u64)> {
        match self.hit_doc.get(prim, &PropertyKey::Parent) {
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

/// Whether `id` names the heading of a component card.
const fn is_card(id: u32) -> bool {
    id >= row::CARD && id < row::NODE && (id - row::CARD).is_multiple_of(row::CARD_STRIDE)
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

    #[test]
    fn a_card_is_named_by_the_first_id_of_its_stride() {
        assert!(is_card(row::CARD));
        assert!(is_card(row::CARD + row::CARD_STRIDE));
        assert!(
            !is_card(row::CARD + 1),
            "a field of a card is not its heading"
        );
        assert!(!is_card(row::NODE), "a hierarchy node is not a card");
        assert!(!is_card(row::CLOSE));
    }
}
