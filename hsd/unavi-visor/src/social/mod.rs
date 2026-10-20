//! The Social lens: a nameplate over each person the viewer is looking at.
//!
//! Three plates are made once and handed from person to person as attention
//! moves, so looking about never builds or frees a surface.

mod plate;

use std::hash::{
    DefaultHasher,
    Hash,
    Hasher,
};

use wired_guest::math::{
    Transform,
    Vec2,
    Vec3,
};

use self::plate::{
    Facts,
    Plate,
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
    wired::perception::perception::{
        self,
        Agent,
    },
};

/// Metres around the viewer searched for people.
const RADIUS: f32 = 16.0;

/// Fixed ticks between rereads of what a plate says, about half a second.
const FACTS_TICKS: u32 = 32;

/// Metres between the head bone and the bottom of the plate.
const OVER_HEAD: f32 = 0.35;

/// Metres above the body a plate stands while the avatar has not loaded.
const OVER_BODY: f32 = 2.0;

/// The size a plate is taken to be before it has text, so attention can
/// measure to where one would stand.
const NOMINAL: Vec2 = Vec2::new(0.6, 0.2);

/// A user, by a hash of their DID.
type Key = u64;

fn key(user: &str) -> Key {
    let mut hasher = DefaultHasher::new();
    user.hash(&mut hasher);
    hasher.finish()
}

/// A person worth a plate.
struct Found {
    key:    Key,
    user:   String,
    /// Where the bottom of their plate stands.
    at:     Vec3,
    /// Where they are, which attention measures range to.
    centre: Vec3,
}

fn found(agent: Agent) -> Found {
    let (centre, at) = agent.head.map_or_else(
        || {
            let body = agent.body.translation;
            (
                body + Vec3::Y * (OVER_BODY / 2.0),
                body + Vec3::Y * OVER_BODY,
            )
        },
        |head| (head.translation, head.translation + Vec3::Y * OVER_HEAD),
    );
    Found {
        key: key(&agent.user),
        user: agent.user,
        at,
        centre,
    }
}

fn plate_lines(plate: &Plate) -> Vec<PlacardLine> {
    plate
        .line
        .iter()
        .map(|text| PlacardLine {
            text: text.clone(),
            tone: Tone::Muted,
            icon: None,
        })
        .collect()
}

/// What `user` is known as, read now.
fn describe(user: &str) -> Plate {
    let profile = perception::profile(user).ok().flatten();
    let rung = perception::rung(user).ok();
    plate::describe(&Facts {
        user,
        name: profile.as_ref().and_then(|profile| profile.name.as_deref()),
        description: profile
            .as_ref()
            .and_then(|profile| profile.description.as_deref()),
        rung,
    })
}

struct Slot {
    placard: Placard,
    key:     Option<Key>,
    /// What the plate says now.
    text:    Option<Plate>,
    /// Whether the text has been tried since the slot was given its person or
    /// since the facts were last reread, so a refused text is not retried and
    /// logged every tick.
    tried:   bool,
    at:      Option<Vec3>,
    /// Its laid-out size, read back once its text is set.
    size:    Vec2,
    shown:   bool,
}

impl Slot {
    fn claim(&mut self, key: Option<Key>) {
        self.key = key;
        self.text = None;
        self.tried = false;
        self.at = None;
        self.size = NOMINAL;
    }

    fn hide(&mut self) {
        self.claim(None);
        if self.shown {
            self.placard.set_shown(false);
            self.shown = false;
        }
    }

    fn write(&mut self, user: &str) {
        let text = describe(user);
        if self.text.as_ref() == Some(&text) {
            return;
        }
        match self
            .placard
            .set_text(&text.title, text.kicker.as_deref(), &plate_lines(&text))
        {
            Ok(()) => {
                self.text = Some(text);
                self.size = self.placard.size();
            }
            Err(err) => eprintln!("visor: nameplate text: {err:?}"),
        }
    }

    fn stand(&mut self, at: Vec3) {
        if self.at != Some(at) {
            self.placard.set_position(at);
            self.at = Some(at);
        }
    }
}

pub struct Social {
    slots:    Vec<Slot>,
    facts_in: u32,
    /// Whether a failed read has been reported, so a broken host is said once.
    warned:   bool,
}

impl Social {
    pub fn new() -> anyhow::Result<Self> {
        let slots = (0..attention::CAP)
            .map(|_| {
                Ok(Slot {
                    placard: Placard::new()?,
                    key:     None,
                    text:    None,
                    tried:   false,
                    at:      None,
                    size:    NOMINAL,
                    shown:   false,
                })
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(Self {
            slots,
            facts_in: 0,
            warned: false,
        })
    }

    /// Moves attention to whoever the viewer is looking at, while the lens is
    /// worn. Unworn, or with no eye, nobody is attended and every plate is
    /// put away.
    pub fn fixed_update(&mut self, worn: bool, eye: Option<&Transform>) {
        let eye = eye.filter(|_| worn);
        let found = if eye.is_some() {
            self.people()
        } else {
            Vec::new()
        };
        let held: Vec<_> = self.slots.iter().filter_map(|slot| slot.key).collect();
        let attended = eye.map_or_else(Vec::new, |eye| {
            let candidates: Vec<_> = found
                .iter()
                .map(|person| {
                    let size = self
                        .slots
                        .iter()
                        .find(|slot| slot.key == Some(person.key))
                        .map_or(NOMINAL, |slot| slot.size);
                    Candidate {
                        key:    person.key,
                        centre: person.centre,
                        anchor: person.at + Vec3::Y * (size.y / 2.0),
                    }
                })
                .collect();
            attention::PEOPLE.attend(
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
        for slot in &mut self.slots {
            let Some(person) = slot
                .key
                .and_then(|key| found.iter().find(|person| person.key == key))
            else {
                continue;
            };
            if reread || !slot.tried {
                slot.tried = true;
                slot.write(&person.user);
            }
            slot.stand(person.at);
            if slot.text.is_some() && !slot.shown {
                slot.placard.set_shown(true);
                slot.shown = true;
            }
        }
    }

    /// Keeps each plate over its person every frame. Attention moves on fixed
    /// ticks, but a plate stood only then steps behind a moving head.
    pub fn update(&mut self) {
        if self.slots.iter().all(|slot| slot.key.is_none()) {
            return;
        }
        let found = self.people();
        for slot in &mut self.slots {
            if let Some(person) = slot
                .key
                .and_then(|key| found.iter().find(|person| person.key == key))
            {
                slot.stand(person.at);
            }
        }
    }

    fn people(&mut self) -> Vec<Found> {
        match perception::agents(RADIUS) {
            Ok(agents) => agents.into_iter().map(found).collect(),
            Err(err) => {
                if !self.warned {
                    eprintln!("visor: agents: {err:?}");
                    self.warned = true;
                }
                Vec::new()
            }
        }
    }
}
