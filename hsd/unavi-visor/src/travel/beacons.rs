//! Tags on the loose beacons the viewer looks at, naming where each leads.
//!
//! Beacons and the sockets they sit in are heard on the beacon protocol's
//! channels, which a global listener hears from every beacon and gate on this
//! peer. Sockets are heard only to know which beacons are seated, and a
//! seated beacon takes no tag.

use unavi_lens::DocumentId;
use wired_guest::{
    beacon::{
        Announce,
        BEACON_CHANNEL,
        SOCKET_CHANNEL,
        Socket,
    },
    color::{
        document_seed,
        generate_color,
    },
    math::{
        Color,
        Transform,
        Vec2,
        Vec3,
    },
};

use super::{
    beside::beside,
    content::{
        self,
        Content,
        Facts,
    },
    frame::Placement,
    lines::tag_lines,
    listing::Listings,
};
use crate::{
    attention::{
        self,
        Candidate,
        Reach,
    },
    clearance::Clearance,
    slots,
    unavi::vui::api::Placard,
    wired::{
        event::messaging::{
            self,
            MessageSubscription,
            Scope,
        },
        perception::perception,
        scene::document::{
            Document,
            open_document,
        },
    },
};

/// Beacons are looked at from within arm's reach or a step or two beyond.
const REACH: Reach = Reach {
    enter: 3.0,
    keep:  3.8,
};

/// Metres from the eye past which a heard beacon or socket is forgotten until
/// it announces again.
const FORGET_RADIUS: f32 = 12.0;

/// The size a tag is taken to be before it has text.
const NOMINAL: Vec2 = Vec2::new(0.84, 0.3);

/// Fixed ticks between rereads of what a tag says.
const FACTS_TICKS: u32 = 16;

/// A beacon, by its document.
type Key = DocumentId;

/// A loose beacon in range, measured this tick.
struct Seen {
    key:   Key,
    /// Where it is, in the visor's frame.
    at:    Vec3,
    /// The space it leads to.
    space: DocumentId,
}

struct HeardBeacon {
    doc:      Document,
    announce: Announce,
}

struct HeardSocket {
    doc:    Document,
    socket: Socket,
}

struct Slot {
    placard:  Placard,
    key:      Option<Key>,
    text:     Option<Content>,
    /// Where it stands and what its tether reaches for, as last sent.
    at:       Option<Placement>,
    /// The accent last sent, once one has been since the slot was claimed.
    accent:   Option<Color>,
    size:     Vec2,
    shown:    bool,
    facts_in: u32,
}

impl Slot {
    /// Sets the tag's text when it differs from what it says now.
    fn retext(&mut self, text: Content) {
        if self.text.as_ref() == Some(&text) {
            return;
        }
        match self
            .placard
            .set_text(&text.title, Some(&text.kicker), &tag_lines(&text))
        {
            Ok(()) => {
                self.size = self.placard.size();
                self.text = Some(text);
            }
            Err(err) => eprintln!("visor: beacon tag text: {err:?}"),
        }
    }

    /// Stands it beside `target` as seen from `eye`, sending only what moved.
    fn stand(&mut self, eye: Vec3, target: Vec3, clearance: &mut Clearance) {
        let mut placed = beside(eye, target, self.size);
        let own = self.key;
        placed.at = clearance.clear(eye, placed.at, self.size, own.as_slice());
        if self.at != Some(placed) {
            self.placard.set_position(placed.at);
            self.placard.set_tether(placed.tether);
            self.at = Some(placed);
        }
    }

    fn hide(&mut self) {
        self.key = None;
        self.text = None;
        self.facts_in = 0;
        self.at = None;
        self.accent = None;
        self.size = NOMINAL;
        if self.shown {
            self.placard.set_shown(false);
            self.shown = false;
        }
    }
}

pub struct Beacons {
    rx:        MessageSubscription,
    heard:     Vec<HeardBeacon>,
    sockets:   Vec<HeardSocket>,
    slots:     Vec<Slot>,
    clearance: Clearance,
}

impl Beacons {
    pub fn new() -> anyhow::Result<Self> {
        let rx = messaging::listen(
            &[BEACON_CHANNEL.to_owned(), SOCKET_CHANNEL.to_owned()],
            None,
            Scope::Global,
        )?;
        let slots = (0..attention::CAP)
            .map(|_| {
                Ok(Slot {
                    placard:  Placard::new()?,
                    key:      None,
                    text:     None,
                    at:       None,
                    accent:   None,
                    size:     NOMINAL,
                    shown:    false,
                    facts_in: 0,
                })
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            rx,
            heard: Vec::new(),
            sockets: Vec::new(),
            slots,
            clearance: Clearance::new()?,
        })
    }

    /// Tags what the viewer looks at, while the lens is worn. Unworn, or with
    /// no eye, every tag is put away.
    pub const fn set_shell(&mut self, shell: Option<DocumentId>) {
        self.clearance.set_shell(shell);
    }

    pub fn fixed_update(&mut self, eye: Option<&Transform>, listings: &Listings) {
        let Some(eye) = eye else {
            for slot in &mut self.slots {
                slot.hide();
            }
            return;
        };
        self.hear();
        let seen = self.measure(eye.translation);

        let candidates: Vec<_> = seen
            .iter()
            .map(|seen| {
                let size = self
                    .slots
                    .iter()
                    .find(|slot| slot.key == Some(seen.key))
                    .map_or(NOMINAL, |slot| slot.size);
                Candidate {
                    key:    seen.key,
                    centre: seen.at,
                    anchor: beside(eye.translation, seen.at, size).middle(size),
                }
            })
            .collect();
        let held: Vec<_> = self.slots.iter().filter_map(|slot| slot.key).collect();
        let attended = REACH.attend(
            eye.translation,
            eye.rotation * Vec3::NEG_Z,
            &candidates,
            &held,
        );

        let mut keys: Vec<_> = self.slots.iter().map(|slot| slot.key).collect();
        let changes = slots::assign(&mut keys, &attended);
        for index in changes.released {
            self.slots[index].hide();
        }
        for index in changes.claimed {
            self.slots[index].key = keys[index];
        }

        for slot in &mut self.slots {
            let Some(seen) = slot
                .key
                .and_then(|key| seen.iter().find(|seen| seen.key == key))
            else {
                continue;
            };
            if slot.facts_in > 0 {
                slot.facts_in -= 1;
            } else {
                slot.facts_in = FACTS_TICKS;
                slot.retext(describe(seen.space, listings));
            }
            let accent = generate_color(document_seed(seen.space));
            if slot.accent != Some(accent) {
                slot.placard.set_accent(Some(accent));
                slot.accent = Some(accent);
            }
            slot.stand(eye.translation, seen.at, &mut self.clearance);
            if slot.text.is_some() && !slot.shown {
                slot.placard.set_shown(true);
                slot.shown = true;
            }
        }
    }

    /// Keeps each tag by its beacon every frame. Attention moves on fixed
    /// ticks, but a tag stood only then steps behind a moving beacon.
    pub fn update(&mut self, eye: Option<&Transform>) {
        let Some(eye) = eye else {
            return;
        };
        for slot in &mut self.slots {
            let Some(heard) = slot
                .key
                .and_then(|key| self.heard.iter().find(|heard| heard.doc.id() == key))
            else {
                continue;
            };
            if let Ok(body) = heard.doc.world_transform(heard.announce.body) {
                slot.stand(eye.translation, body.translation, &mut self.clearance);
            }
        }
    }

    fn hear(&mut self) {
        for message in self.rx.drain(64) {
            let Some(sender) = message.sender else {
                continue;
            };
            if message.channel == BEACON_CHANNEL {
                let Some(announce) = Announce::decode(&message.payload) else {
                    continue;
                };
                if let Some(heard) = self.heard.iter_mut().find(|h| h.doc.id() == sender) {
                    heard.announce = announce;
                } else if let Some(doc) = open(sender) {
                    self.heard.push(HeardBeacon { doc, announce });
                }
            } else {
                let Some(socket) = Socket::decode(&message.payload) else {
                    continue;
                };
                if let Some(heard) = self
                    .sockets
                    .iter_mut()
                    .find(|h| h.doc.id() == sender && h.socket.seat == socket.seat)
                {
                    heard.socket = socket;
                } else if let Some(doc) = open(sender) {
                    self.sockets.push(HeardSocket { doc, socket });
                }
            }
        }
    }

    /// Every heard beacon that is neither seated nor held, forgetting what is
    /// gone or far.
    fn measure(&mut self, eye: Vec3) -> Vec<Seen> {
        let mut seen = Vec::new();
        let seated: Vec<_> = self
            .sockets
            .iter()
            .filter_map(|heard| heard.socket.occupant)
            .collect();
        self.heard.retain(|heard| {
            let Ok(body) = heard.doc.world_transform(heard.announce.body) else {
                return false;
            };
            if body.translation.distance(eye) > FORGET_RADIUS {
                return false;
            }
            if !heard.announce.held && !seated.contains(&heard.doc.id()) {
                seen.push(Seen {
                    key:   heard.doc.id(),
                    at:    body.translation,
                    space: heard.announce.space,
                });
            }
            true
        });
        self.sockets.retain(|heard| {
            heard
                .doc
                .world_transform(heard.socket.seat)
                .is_ok_and(|seat| seat.translation.distance(eye) <= FORGET_RADIUS)
        });
        seen
    }
}

/// The sender of a message, when it is loaded here.
fn open(sender: DocumentId) -> Option<Document> {
    match open_document(sender) {
        Ok(doc) => doc,
        Err(err) => {
            eprintln!("visor: open {sender:?}: {err:?}");
            None
        }
    }
}

fn describe(space: DocumentId, listings: &Listings) -> Content {
    let meta_name = perception::meta(space)
        .ok()
        .flatten()
        .and_then(|meta| meta.name);
    content::describe(&Facts {
        listed_title: listings.title(space),
        meta_name:    meta_name.as_deref(),
        destination:  space,
        occupants:    listings.occupants(space),
        linked:       true,
        author:       None,
    })
}
