//! Visor: the lenses worn over the world.
//!
//! The lenses belong to the visor script, which announces them. This branch
//! only mirrors that announcement as toggle motes and sends back what the user
//! chooses, so a lens added to the visor shows up here with no change.
//!
//! Both channels are open to any script, so an announcement is believed only
//! from a document the local user authors.

mod sync;

use unavi_lens::{
    ASK,
    DocumentId,
    LENSES,
    Lens,
    WEAR,
    Wear,
    intake::{
        Heard,
        Host,
        Intake,
    },
};

use self::sync::plan;
use crate::{
    glyph,
    palette,
    unavi::vui::api::{
        Kind,
        Mote,
    },
    wired::{
        event::messaging::{
            self,
            MessageSubscription,
            Scope,
        },
        peer::authority,
        scene::document::open_document,
    },
};

/// The halo's bindings to the event queue.
struct Events;

impl Host for Events {
    type Subscription = MessageSubscription;

    fn listen(&self, senders: Option<&[DocumentId]>) -> Result<MessageSubscription, String> {
        messaging::listen(&[LENSES.to_owned()], senders, Scope::Global)
            .map_err(|err| format!("{err:?}"))
    }

    fn drain(&self, subscription: &MessageSubscription, max: u32) -> Vec<Heard> {
        subscription
            .drain(max)
            .into_iter()
            .map(|message| Heard {
                sender:  message.sender,
                channel: message.channel,
                payload: message.payload,
            })
            .collect()
    }

    fn is_mine(&self, document: &DocumentId) -> Option<bool> {
        unavi_lens::authored(open_document(*document).ok().flatten(), |doc| {
            authority::is_author(doc).ok()
        })
    }
}

struct Entry {
    id:   String,
    mote: Mote,
}

#[derive(Default)]
pub struct Visor {
    intake: Intake<MessageSubscription>,
    /// One per announced lens, in announced order.
    lenses: Vec<Entry>,
}

impl Visor {
    /// Starts hearing the visor, so its first announcement is not missed.
    pub fn start(&mut self) {
        self.intake.start(&Events);
    }

    /// Asks the visor to announce its lenses again. Called at start and when
    /// the branch opens, so what is shown is what the visor holds now.
    pub fn ask() {
        if let Err(err) = messaging::emit(ASK, &[], None, Scope::Global) {
            eprintln!("halo: ask: {err:?}");
        }
    }

    /// Rebuilds the lens motes under `parent` from the latest announcement
    /// heard. A lens already shown keeps its mote.
    pub fn fixed_update(&mut self, parent: &Mote) {
        let mut latest = None;
        for heard in self.intake.poll(&Events) {
            if heard.channel != LENSES {
                continue;
            }
            match unavi_lens::decode_lenses(&heard.payload) {
                Ok(lenses) => latest = Some(lenses),
                Err(err) => eprintln!("halo: bad lenses: {err}"),
            }
        }
        if let Some(lenses) = latest {
            self.sync(parent, &lenses);
        }
    }

    /// Whether `mote` is a lens, and tells the visor to wear or remove it if
    /// it is. The mote has already flipped itself, so it is what the user
    /// chose.
    pub fn wear(&self, mote: &Mote) -> bool {
        let Some(entry) = self.lenses.iter().find(|entry| entry.mote.is(mote)) else {
            return false;
        };
        let request = Wear {
            id:   entry.id.clone(),
            worn: mote.active(),
        };
        match unavi_lens::encode_wear(&request) {
            Ok(bytes) => {
                if let Err(err) = messaging::emit(WEAR, &bytes, None, Scope::Global) {
                    eprintln!("halo: wear: {err:?}");
                }
            }
            Err(err) => eprintln!("halo: encode wear: {err}"),
        }
        true
    }

    fn sync(&mut self, parent: &Mote, announced: &[Lens]) {
        let have: Vec<&str> = self.lenses.iter().map(|entry| entry.id.as_str()).collect();
        let plan = plan(&have, announced);

        let mut next = Vec::with_capacity(plan.show.len());
        for lens in plan.show {
            let existing = self.lenses.iter().position(|entry| entry.id == lens.id);
            let entry = match existing {
                Some(index) => self.lenses.swap_remove(index),
                None => match build(lens) {
                    Ok(mote) => Entry {
                        id: lens.id.clone(),
                        mote,
                    },
                    Err(err) => {
                        eprintln!("halo: lens mote for '{}': {err:?}", lens.name);
                        continue;
                    }
                },
            };
            describe(&entry.mote, lens);
            next.push(entry);
        }
        // What is left in `self.lenses` is what the plan dropped.
        for gone in &self.lenses {
            parent.remove_child(&gone.mote);
        }
        // A level appends and cannot insert, so putting the motes back in
        // announced order is what keeps their slots from shuffling.
        for entry in &next {
            parent.add_child(&entry.mote);
        }
        self.lenses = next;
    }
}

fn build(lens: &Lens) -> anyhow::Result<Mote> {
    let mote = Mote::new(Kind::Toggle, &lens.name)?;
    mote.set_tint(Some(palette::SELF));
    let mark = glyph::lens(&lens.id);
    if let Err(err) = mote.set_glyph(Some(&glyph::from_rows(mark))) {
        eprintln!("halo: lens glyph for '{}': {err:?}", lens.name);
    }
    Ok(mote)
}

/// Brings a mote up to what the visor says, touching only what differs so an
/// unchanged lens redraws nothing.
fn describe(mote: &Mote, lens: &Lens) {
    if mote.label() != lens.name
        && let Err(err) = mote.set_label(&lens.name)
    {
        eprintln!("halo: lens label for '{}': {err:?}", lens.name);
    }
    if mote.description().as_deref() != Some(lens.description.as_str())
        && let Err(err) = mote.set_description(&lens.description)
    {
        eprintln!("halo: lens description for '{}': {err:?}", lens.name);
    }
    if mote.active() != lens.worn {
        mote.set_active(lens.worn);
    }
}
