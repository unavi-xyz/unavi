//! The Travel lens: a placard over each portal the viewer is looking at.
//!
//! Three placards are made once and handed from portal to portal as attention
//! moves, so looking about never builds or frees a surface.

mod attention;
mod content;
mod listing;
mod slots;

use unavi_lens::DocumentId;
use wired_guest::math::{
    Transform,
    Vec3,
};

use self::{
    attention::Candidate,
    content::{
        Content,
        Facts,
    },
    listing::Listings,
};
use crate::{
    unavi::vui::api::Placard,
    wired::{
        peer::authority,
        perception::perception::{
            self,
            Portal,
            PortalState,
            TrustRung,
        },
        scene::document::open_document,
    },
};

/// Metres around the viewer searched for portals.
const RADIUS: f32 = 16.0;

/// Metres above a portal's top edge its placard stands.
const LIFT: f32 = 0.08;

/// Fixed ticks between rereads of what a placard says, about half a second.
const FACTS_TICKS: u32 = 32;

/// A portal, by its prim.
type Key = (DocumentId, (u64, u64));

/// A portal worth a placard.
struct Found {
    key:         Key,
    destination: DocumentId,
    linked:      bool,
    anchor:      Vec3,
    centre:      Vec3,
}

struct Slot {
    placard: Placard,
    key:     Option<Key>,
    /// What the placard says now.
    text:    Option<Content>,
    /// Whether the text has been tried since the slot was given its portal or
    /// since the facts were last reread, so a refused text is not retried and
    /// logged every tick.
    tried:   bool,
    at:      Option<Vec3>,
    shown:   bool,
}

impl Slot {
    fn claim(&mut self, key: Option<Key>) {
        self.key = key;
        self.text = None;
        self.tried = false;
    }

    fn hide(&mut self) {
        self.claim(None);
        if self.shown {
            self.placard.set_shown(false);
            self.shown = false;
        }
    }
}

pub struct Travel {
    slots:    Vec<Slot>,
    listings: Listings,
    facts_in: u32,
    /// Whether a failed read has been reported, so a broken host is said once.
    warned:   bool,
}

impl Travel {
    pub fn new() -> anyhow::Result<Self> {
        let slots = (0..attention::CAP)
            .map(|_| {
                Ok(Slot {
                    placard: Placard::new()?,
                    key:     None,
                    text:    None,
                    tried:   false,
                    at:      None,
                    shown:   false,
                })
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            slots,
            listings: Listings::default(),
            facts_in: 0,
            warned: false,
        })
    }

    /// Moves attention to what the viewer is looking at, while the lens is
    /// worn. Unworn, or with no eye, nothing is attended and every placard is
    /// put away.
    pub fn fixed_update(&mut self, worn: bool, eye: Option<&Transform>) {
        let eye = eye.filter(|_| worn);
        let found = if eye.is_some() {
            self.listings.fixed_update();
            self.portals()
        } else {
            Vec::new()
        };
        let held: Vec<_> = self.slots.iter().filter_map(|slot| slot.key).collect();
        let attended = eye.map_or_else(Vec::new, |eye| {
            let candidates: Vec<_> = found
                .iter()
                .map(|portal| Candidate {
                    key:    portal.key,
                    centre: portal.centre,
                    anchor: portal.anchor,
                })
                .collect();
            attention::attend(
                eye.translation,
                eye.rotation * Vec3::NEG_Z,
                &candidates,
                &held,
            )
        });

        let mut keys: Vec<_> = self.slots.iter().map(|slot| slot.key).collect();
        let changes = slots::assign(&mut keys, &attended);
        for index in changes.released {
            self.slots[index].hide();
        }
        for index in changes.claimed {
            self.slots[index].claim(keys[index]);
        }

        let reread = self.facts_in == 0;
        self.facts_in = if reread {
            FACTS_TICKS
        } else {
            self.facts_in - 1
        };
        for index in 0..self.slots.len() {
            let Some(portal) = self.slots[index]
                .key
                .and_then(|key| found.iter().find(|portal| portal.key == key))
            else {
                continue;
            };
            let slot = &mut self.slots[index];
            if slot.at != Some(portal.anchor) {
                slot.placard.set_position(portal.anchor);
                slot.at = Some(portal.anchor);
            }
            if reread || !slot.tried {
                slot.tried = true;
                let text = describe(&self.listings, portal);
                if slot.text.as_ref() != Some(&text) {
                    match slot.placard.set_text(&text.title, &text.lines) {
                        Ok(()) => slot.text = Some(text),
                        Err(err) => eprintln!("visor: placard text: {err:?}"),
                    }
                }
            }
            if slot.text.is_some() && !slot.shown {
                slot.placard.set_shown(true);
                slot.shown = true;
            }
        }
    }

    /// The portals in range that go somewhere.
    fn portals(&mut self) -> Vec<Found> {
        match perception::portals(RADIUS) {
            Ok(portals) => portals.iter().filter_map(found).collect(),
            Err(err) => {
                if !self.warned {
                    eprintln!("visor: portals: {err:?}");
                    self.warned = true;
                }
                Vec::new()
            }
        }
    }
}

fn found(portal: &Portal) -> Option<Found> {
    let destination = portal.destination?;
    if matches!(portal.state, PortalState::Closed) {
        return None;
    }
    let at = &portal.transform;
    let top = Vec3::new(0.0, portal.height / 2.0, 0.0);
    Some(Found {
        key: (portal.prim.document, portal.prim.prim),
        destination,
        linked: portal.linked,
        anchor: at.translation + at.rotation * (at.scale * top) + Vec3::Y * LIFT,
        centre: at.translation,
    })
}

fn describe(listings: &Listings, portal: &Found) -> Content {
    let meta_name = perception::meta(portal.destination)
        .ok()
        .flatten()
        .and_then(|meta| meta.name);
    content::describe(&Facts {
        listed_title: listings.title(portal.destination),
        meta_name:    meta_name.as_deref(),
        destination:  portal.destination,
        occupants:    listings.occupants(portal.destination),
        linked:       portal.linked,
        author:       author_rung(portal.destination),
    })
}

/// The local rung of whoever authors `destination`, when it is loaded and
/// someone has proven themselves its author.
fn author_rung(destination: DocumentId) -> Option<TrustRung> {
    let doc = open_document(destination).ok().flatten()?;
    let author = authority::author(&doc).ok().flatten()?;
    perception::rung(&author).ok()
}
