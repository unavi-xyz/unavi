//! Nav: the spaces you can go to.
//!
//! A level that opens as a grid, so the listing gets pagination, attention and
//! placards for free.

use std::str::FromStr;

use blake3::Hash;
use wired_guest::{
    beacon::{
        PLANTED_KEY,
        now_ms,
        planted_bytes,
    },
    color::generate_color,
    math::{
        Transform,
        Vec3,
    },
};

use crate::{
    glyph,
    unavi::{
        host::node_storage::{
            self,
            Entry,
            PendingEntries,
        },
        vui::api::{
            Kind,
            Landing,
            Mote,
        },
    },
    wired::{
        portal::portals::{
            current_space,
            travel,
        },
        scene::{
            document::{
                Anchor,
                Document,
                Layer,
                copy_document,
                open_document,
                script_document,
            },
            properties::{
                Property,
                PropertyKey,
            },
        },
    },
};

/// Prefix of a registry's active-spaces view; listing it picks out activity
/// and ignores the rest. The other views a registry publishes share the
/// namespace list and answer this prefix with nothing.
const ACTIVE_PREFIX: &str = "active/";
/// The documented bound on `list-entries`.
const LIST_LIMIT: u32 = 256;

/// The authored prim referencing the document every beacon is copied from,
/// kept at zero scale so the template itself never shows.
const TEMPLATE_PRIM_NAME: &str = "beacon_template";

/// What letting go of a mote did, as far as Nav is concerned.
pub enum Planting {
    /// The mote is not one of Nav's beacons.
    NotOurs,
    /// A beacon is down.
    Planted,
    /// It was Nav's beacon, but none could be put down; the mote says so.
    Failed,
}

/// A space the registries say has people in it.
struct Space {
    /// For travelling, and the beacon's own colour seed.
    ns:     (u64, u64, u64, u64),
    /// The same, as the registry wrote it, which is what a beacon is named.
    hex:    String,
    /// Who is in it, as the description reads when you are not.
    people: String,
    /// Whether the user is in it now.
    here:   bool,
    group:  Mote,
    travel: Mote,
    beacon: Mote,
}

impl Space {
    /// Sets whether the user is in this space, rewriting what that changes.
    ///
    /// Travel leaves the level while you are here: a ring that fills to take
    /// you where you already stand would only be a trap, and the one thing a
    /// space you are in offers is its beacon. It comes back ahead of the
    /// beacon so the slots keep their order.
    fn set_here(&mut self, here: bool) {
        if self.here == here {
            return;
        }
        self.here = here;
        let text = if here { HERE } else { &self.people };
        if let Err(err) = self.group.set_description(text) {
            eprintln!("halo: space description: {err:?}");
        }
        // The pin says it at a glance, before anyone reads a placard.
        let mark = if here { &glyph::HERE } else { &glyph::BEACON };
        if let Err(err) = self.group.set_glyph(Some(&glyph::from_rows(mark))) {
            eprintln!("halo: space glyph: {err:?}");
        }
        if here {
            self.group.remove_child(&self.travel);
        } else {
            self.group.remove_child(&self.beacon);
            let _ = self.group.add_child(&self.travel);
            let _ = self.group.add_child(&self.beacon);
        }
    }
}

/// Lists the spaces currently occupied per the registries this app follows.
#[derive(Default)]
pub struct Nav {
    lists:   Vec<PendingEntries>,
    spaces:  Vec<Space>,
    beacons: Vec<Document>,
    /// The space the user was last seen in.
    here:    Option<(u64, u64, u64, u64)>,
}

impl Nav {
    /// Asks every registry what is live. Called when the branch opens, so a
    /// halo that is never opened costs nothing.
    pub fn refresh(&mut self) {
        let registries = match node_storage::registries() {
            Ok(registries) => registries,
            Err(err) => {
                eprintln!("halo: registries: {err:?}");
                return;
            }
        };
        self.lists = registries
            .into_iter()
            .filter_map(|registry| {
                node_storage::list_entries(registry, ACTIVE_PREFIX, LIST_LIMIT)
                    .inspect_err(|err| eprintln!("halo: list_entries: {err:?}"))
                    .ok()
            })
            .collect();
    }

    /// Hangs a mote under `parent` for each space that has appeared.
    ///
    /// Sorted by the registry's own rank and then by namespace, so the order
    /// is the same every frame however the listings interleave — slot order is
    /// position, and position is muscle memory.
    pub fn fixed_update(&mut self, parent: &Mote) {
        self.follow_user();

        let mut found = Vec::new();
        for index in (0..self.lists.len()).rev() {
            let Some(result) = self.lists[index].poll() else {
                continue;
            };
            self.lists.remove(index);
            match result {
                Ok(entries) => found.extend(entries.iter().filter_map(entry)),
                Err(err) => eprintln!("halo: registry active-space list error: {err:?}"),
            }
        }
        if found.is_empty() {
            return;
        }

        found.sort_by(|a, b| a.rank.cmp(&b.rank).then_with(|| a.hex.cmp(&b.hex)));
        for listing in found {
            if self.spaces.iter().any(|space| space.hex == listing.hex) {
                continue;
            }
            let Some(mut space) = build(&listing) else {
                continue;
            };
            space.set_here(self.here == Some(space.ns));
            parent.add_child(&space.group);
            self.spaces.push(space);
        }
    }

    /// Marks the space the user is in. A user between spaces, or a host that
    /// will not say, is in none.
    fn follow_user(&mut self) {
        let here = current_space().ok().flatten();
        if here == self.here {
            return;
        }
        self.here = here;
        for space in &mut self.spaces {
            space.set_here(here == Some(space.ns));
        }
    }

    /// Travels to whichever space's cast just filled.
    pub fn cast(&self, mote: &Mote) -> bool {
        let Some(space) = self.spaces.iter().find(|space| space.travel.is(mote)) else {
            return false;
        };
        if let Err(err) = travel(space.ns) {
            eprintln!("halo: travel failed: {err:?}");
            space.travel.notify("Couldn't travel there.");
        }
        true
    }

    /// Puts a beacon for a space where its mote was let go.
    ///
    /// A beacon is a document of its own rather than an instance under one of
    /// our prims: beacons sync per document, and an instance has no document
    /// to sync.
    pub fn plant(&mut self, mote: &Mote, landing: Landing) -> Planting {
        let Some(space) = self.spaces.iter().find(|space| space.beacon.is(mote)) else {
            return Planting::NotOurs;
        };
        match mint(&space.hex, landing.at) {
            Ok(beacon) => {
                self.beacons.push(beacon);
                Planting::Planted
            }
            Err(err) => {
                eprintln!("halo: could not put down a beacon: {err:?}");
                space.beacon.notify("Couldn't put the beacon down.");
                Planting::Failed
            }
        }
    }
}

/// One space as a registry reported it.
struct Listing {
    rank:      u32,
    hex:       String,
    ns:        (u64, u64, u64, u64),
    occupants: u32,
    idle_secs: u64,
    /// The listing title as the registry wrote it; may be blank.
    title:     String,
}

/// Decodes an `active/<rank>/<space-hex>` entry.
///
/// The value carries occupancy and the listing title, which is all the view
/// holds: the rest of a listing lives in the registry's other views behind a
/// payload a guest cannot decode.
fn entry(entry: &Entry) -> Option<Listing> {
    let mut parts = entry.key.strip_prefix(ACTIVE_PREFIX)?.split('/');
    let rank = parts.next()?.parse().ok()?;
    let space = parts.next().filter(|space| !space.is_empty())?;
    let (occupants, idle_secs, title) = postcard::from_bytes(&entry.value).ok()?;

    Some(Listing {
        rank,
        ns: document_id_from_hex(space)?,
        hex: space.to_owned(),
        occupants,
        idle_secs,
        title,
    })
}

/// A space, and the two things you can do with one.
///
/// Two motes rather than one because the roles say different things: travel is
/// consequential and gets a fill ring, and only an item leaves its slot to be
/// thrown. A mote cannot be both.
///
/// Every space wears the colour its beacon will: the beacon's hue is derived
/// from the space's own id, so the mote you pick out of the halo and the beacon
/// you put down in the world are the same colour, and a grid of spaces
/// separates at a glance rather than reading as one green sheet.
fn build(listing: &Listing) -> Option<Space> {
    let color = generate_color(Hash::from_str(&listing.hex).ok()?);

    let group = Mote::new(Kind::Group, &label(listing)?).ok()?;
    let people = people(listing);
    group.set_description(&people).ok()?;
    group.set_tint(Some(color));
    // The space wears its beacon's mark: in the grid it reads as the marker
    // it is, and opening it still shows the travel and beacon motes beneath.
    // A refused glyph is not a reason to lose the whole space.
    if let Err(err) = group.set_glyph(Some(&glyph::from_rows(&glyph::BEACON))) {
        eprintln!("halo: space glyph: {err:?}");
    }

    let travel = Mote::new(Kind::Cast, "Travel").ok()?;
    travel.set_description("Go to this space.").ok()?;
    travel.set_tint(Some(color));

    let beacon = Mote::new(Kind::Item, "Beacon").ok()?;
    beacon.set_description("A marker you can drop here.").ok()?;
    beacon.set_tint(Some(color));
    if let Err(err) = beacon.set_glyph(Some(&glyph::from_rows(&glyph::BEACON))) {
        eprintln!("halo: beacon glyph: {err:?}");
    }

    group.add_child(&travel);
    group.add_child(&beacon);

    Some(Space {
        ns: listing.ns,
        hex: listing.hex.clone(),
        people,
        here: false,
        group,
        travel,
        beacon,
    })
}

/// The most characters a space's label shows, which is what fits under a mote.
const LABEL_MAX: usize = 24;

/// The space's title, shortened to fit under its mote; the head of its
/// namespace when the listing has no title.
fn label(listing: &Listing) -> Option<String> {
    let title = listing.title.trim();
    if title.is_empty() {
        return listing.hex.get(..8).map(str::to_owned);
    }
    if title.chars().count() <= LABEL_MAX {
        return Some(title.to_owned());
    }
    let mut label: String = title.chars().take(LABEL_MAX - 1).collect();
    label.push('…');
    Some(label)
}

/// What a space you are in says instead of its occupancy.
const HERE: &str = "You are here";

/// Who is in a space, and how long it has been quiet once that is worth
/// saying. Only occupancy: the title is already the label, and anything about
/// being "here" is for the one space you are in.
fn people(listing: &Listing) -> String {
    let people = match listing.occupants {
        0 => "Empty".to_owned(),
        1 => "1 person".to_owned(),
        count => format!("{count} people"),
    };
    if listing.idle_secs < 60 {
        people
    } else {
        format!("{people}, quiet {} min", listing.idle_secs / 60)
    }
}

fn mint(hex: &str, at: Vec3) -> anyhow::Result<Document> {
    let doc = script_document()?;
    let template_prim = crate::find_one(&doc, TEMPLATE_PRIM_NAME)?;
    let Some(Property::Reference(referenced)) = doc.get(template_prim, &PropertyKey::Reference)
    else {
        anyhow::bail!("halo HSD's {TEMPLATE_PRIM_NAME} prim has no reference");
    };
    let template = open_document(referenced)?.ok_or_else(|| {
        anyhow::anyhow!("halo HSD's {TEMPLATE_PRIM_NAME} reference is not loaded")
    })?;

    // Built in full while the document is still parked, so the room sees a
    // beacon appear where it was let go rather than one arriving at the
    // origin and moving.
    // A copy rather than a reference: the beacon's script looks for a prim
    // named for its space, and a referenced document realizes as a child, so
    // the script would be looking in the wrong document.
    let beacon = copy_document(&template)?;
    // The landing is in the halo's frame, but the beacon document sits at
    // the space root: convert before writing, or it plants where the halo's
    // frame sits, not where it was let go.
    beacon.place(Anchor::Space, Transform::IDENTITY)?;
    let to_world = doc.world_frame().unwrap_or(Transform::IDENTITY);
    let prim = beacon.create_prim(Layer::Local, None)?;
    // The beacon script finds itself by a prim named for its space.
    beacon
        .local()
        .set(prim, Property::Name(hex.to_owned()))
        .set(
            prim,
            Property::Custom((PLANTED_KEY.to_owned(), planted_bytes(now_ms()).to_vec())),
        )
        .set(
            prim,
            Property::Transform(to_world * Transform::from_translation(at)),
        )
        .flush()?;
    // Planting runs on this peer alone, so the prim must be committed for
    // anyone else's copy of the beacon to find itself.
    let keys: Vec<_> = beacon
        .keys(prim)
        .into_iter()
        .map(|key| (prim, key))
        .collect();
    beacon.commit(&keys)?;

    Ok(beacon)
}

/// The registry writes a namespace as hex; a `document-id` is its bytes as
/// four little-endian words.
fn document_id_from_hex(hex: &str) -> Option<(u64, u64, u64, u64)> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes: Vec<u8> = hex
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(digits, 16).ok()
        })
        .collect::<Option<_>>()?;
    let bytes: &[u8; 32] = bytes.as_slice().try_into().ok()?;
    Some((
        u64::from_le_bytes(bytes[0..8].try_into().ok()?),
        u64::from_le_bytes(bytes[8..16].try_into().ok()?),
        u64::from_le_bytes(bytes[16..24].try_into().ok()?),
        u64::from_le_bytes(bytes[24..32].try_into().ok()?),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(title: &str) -> Listing {
        Listing {
            rank:      0,
            hex:       "0123456789abcdef".to_owned(),
            ns:        (0, 0, 0, 0),
            occupants: 3,
            idle_secs: 0,
            title:     title.to_owned(),
        }
    }

    #[test]
    fn a_short_title_labels_the_space_trimmed() {
        assert_eq!(label(&listing("  Plaza ")).as_deref(), Some("Plaza"));
    }

    #[test]
    fn a_blank_title_falls_back_to_the_namespace_head() {
        assert_eq!(label(&listing("")).as_deref(), Some("01234567"));
        assert_eq!(label(&listing("   ")).as_deref(), Some("01234567"));
    }

    #[test]
    fn a_long_title_is_cut_to_the_label_width_with_an_ellipsis() {
        let exact = "a".repeat(LABEL_MAX);
        assert_eq!(label(&listing(&exact)), Some(exact));

        let label = label(&listing(&"b".repeat(LABEL_MAX + 1))).expect("label");
        assert_eq!(label.chars().count(), LABEL_MAX);
        assert!(label.ends_with('…'));
    }

    #[test]
    fn a_description_is_occupancy_alone() {
        assert_eq!(people(&listing("A long title")), "3 people");
        assert_eq!(
            people(&Listing {
                occupants: 1,
                ..listing("")
            }),
            "1 person"
        );
        assert_eq!(
            people(&Listing {
                occupants: 0,
                ..listing("")
            }),
            "Empty"
        );
    }

    #[test]
    fn quiet_is_said_only_once_it_is_a_minute_or_more() {
        let at = |idle_secs| Listing {
            idle_secs,
            ..listing("")
        };
        assert_eq!(people(&at(59)), "3 people");
        assert_eq!(people(&at(60)), "3 people, quiet 1 min");
        assert_eq!(people(&at(310)), "3 people, quiet 5 min");
    }
}
