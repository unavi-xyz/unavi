//! The Travel lens: a tag beside each portal the viewer is looking at, tied
//! to the portal by a tether in its rim colour.
//!
//! Three tags are made once and handed from portal to portal as attention
//! moves, so looking about never builds or frees a surface.

mod content;
mod frame;
mod listing;

use unavi_lens::DocumentId;
use wired_guest::math::{
    Color,
    Transform,
    Vec2,
    Vec3,
};

use self::{
    content::{
        Content,
        Facts,
    },
    frame::{
        Frame,
        Placement,
        Side,
    },
    listing::Listings,
};
use crate::{
    attention::{
        self,
        Candidate,
    },
    slots,
    unavi::vui::api::{
        Placard,
        PlacardLine,
        Tone,
    },
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

/// Fixed ticks between rereads of what a placard says, about half a second.
const FACTS_TICKS: u32 = 32;

/// The size a tag is taken to be before it has text, so attention can measure
/// to where one would stand.
const NOMINAL: Vec2 = Vec2::new(0.84, 0.3);

/// A portal, by its prim.
type Key = (DocumentId, (u64, u64));

/// The lines under a tag's title: its facts, muted, and a warning when there
/// is no way back.
fn tag_lines(text: &Content) -> Vec<PlacardLine> {
    let facts = text.facts.iter().map(|facts| PlacardLine {
        text: facts.clone(),
        tone: Tone::Muted,
    });
    let one_way = text.one_way.then(|| PlacardLine {
        text: String::from("One way"),
        tone: Tone::Warning,
    });
    facts.chain(one_way).collect()
}

/// A portal worth a placard.
struct Found {
    key:         Key,
    destination: DocumentId,
    linked:      bool,
    frame:       Frame,
    hue:         Color,
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
    /// Where it stands and what its tether reaches for, as last sent.
    at:      Option<Placement>,
    /// Which post of its portal it stands by, once it has stood by one.
    side:    Option<Side>,
    /// Its laid-out size, read back once its text is set.
    size:    Vec2,
    accent:  Option<Color>,
    shown:   bool,
}

impl Slot {
    fn claim(&mut self, key: Option<Key>) {
        self.key = key;
        self.text = None;
        self.tried = false;
        self.side = None;
        self.at = None;
        self.size = NOMINAL;
    }

    /// Stands it by `portal` as seen from `eye`, sending only what moved.
    fn stand(&mut self, portal: &Found, eye: Vec3) {
        let side = portal.frame.side(eye, self.side);
        self.side = Some(side);
        let placed = portal.frame.place(eye, side, self.size);
        if self.at != Some(placed) {
            self.placard.set_position(placed.at);
            self.placard.set_tether(placed.tether);
            self.at = Some(placed);
        }
        if self.accent != Some(portal.hue) {
            self.placard.set_accent(Some(portal.hue));
            self.accent = Some(portal.hue);
        }
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
                    side:    None,
                    size:    NOMINAL,
                    accent:  None,
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
                .map(|portal| {
                    let held = self.slots.iter().find(|slot| slot.key == Some(portal.key));
                    let size = held.map_or(NOMINAL, |slot| slot.size);
                    let post = portal
                        .frame
                        .side(eye.translation, held.and_then(|slot| slot.side));
                    Candidate {
                        key:    portal.key,
                        centre: portal.frame.centre(),
                        anchor: portal.frame.place(eye.translation, post, size).middle(size),
                    }
                })
                .collect();
            attention::PORTALS.attend(
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
        let Some(eye) = eye else {
            return;
        };
        for index in 0..self.slots.len() {
            let Some(portal) = self.slots[index]
                .key
                .and_then(|key| found.iter().find(|portal| portal.key == key))
            else {
                continue;
            };
            let slot = &mut self.slots[index];
            if reread || !slot.tried {
                slot.tried = true;
                let text = describe(&self.listings, portal);
                if slot.text.as_ref() != Some(&text) {
                    let lines = tag_lines(&text);
                    match slot
                        .placard
                        .set_text(&text.title, Some(&text.kicker), &lines)
                    {
                        Ok(()) => {
                            slot.text = Some(text);
                            slot.size = slot.placard.size();
                        }
                        Err(err) => eprintln!("visor: placard text: {err:?}"),
                    }
                }
            }
            slot.stand(portal, eye.translation);
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
    Some(Found {
        key: (portal.prim.document, portal.prim.prim),
        destination,
        linked: portal.linked,
        frame: Frame::new(&portal.transform, portal.width, portal.height),
        hue: portal.hue,
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
