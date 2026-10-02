//! Nav: the spaces you can go to.
//!
//! A level that opens as a grid, so the listing gets pagination, attention and
//! placards for free.

use std::str::FromStr;

use blake3::Hash;
use wired_guest::{
    color::generate_color,
    math::{
        Transform,
        Vec3,
    },
};

use crate::{
    icon,
    palette,
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
        portal::portals::travel,
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

/// A space the registries say has people in it.
struct Space {
    /// For travelling, and the beacon's own colour seed.
    ns:     (u64, u64, u64, u64),
    /// The same, as the registry wrote it, which is what a beacon is named.
    hex:    String,
    group:  Mote,
    travel: Mote,
    beacon: Mote,
}

/// Lists the spaces currently occupied per the registries this client follows.
#[derive(Default)]
pub struct Nav {
    lists:   Vec<PendingEntries>,
    spaces:  Vec<Space>,
    beacons: Vec<Document>,
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
            let Some(space) = build(&listing) else {
                continue;
            };
            parent.add_child(&space.group);
            self.spaces.push(space);
        }
    }

    /// Travels to whichever space's cast just filled.
    pub fn cast(&self, mote: &Mote) -> bool {
        let Some(space) = self.spaces.iter().find(|space| space.travel.is(mote)) else {
            return false;
        };
        if let Err(err) = travel(space.ns) {
            eprintln!("halo: travel failed: {err:?}");
        }
        true
    }

    /// Puts a beacon for a space where its mote was let go.
    ///
    /// A beacon is a document of its own rather than an instance under one of
    /// our prims: beacons sync per document, and an instance has no document
    /// to sync.
    pub fn plant(&mut self, mote: &Mote, landing: Landing) -> bool {
        let Some(space) = self.spaces.iter().find(|space| space.beacon.is(mote)) else {
            return false;
        };
        match mint(&space.hex, landing.at) {
            Ok(beacon) => self.beacons.push(beacon),
            Err(err) => eprintln!("halo: could not put down a beacon: {err:?}"),
        }
        true
    }
}

/// One space as a registry reported it.
struct Listing {
    rank:      u32,
    hex:       String,
    ns:        (u64, u64, u64, u64),
    occupants: u32,
    idle_secs: u64,
}

/// Decodes an `active/<rank>/<space-hex>` entry.
///
/// The value carries occupancy. There is no display name in this view — that
/// lives in the registry's other views behind a payload a guest cannot decode
/// — so a space is named by the head of its namespace, and the placard says
/// only what is actually known.
fn entry(entry: &Entry) -> Option<Listing> {
    let mut parts = entry.key.strip_prefix(ACTIVE_PREFIX)?.split('/');
    let rank = parts.next()?.parse().ok()?;
    let space = parts.next().filter(|space| !space.is_empty())?;
    let (occupants, idle_secs) = postcard::from_bytes(&entry.value).ok()?;

    Some(Listing {
        rank,
        ns: document_id_from_hex(space)?,
        hex: space.to_owned(),
        occupants,
        idle_secs,
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

    let group = Mote::new(Kind::Group, listing.hex.get(..8)?).ok()?;
    group.describe(&describe(listing)).ok()?;
    group.set_tint(Some(color));
    // The space wears its beacon's form: in the grid it reads as the marker
    // it is, and opening it still shows the travel and beacon motes beneath.
    if let Ok(doc) = script_document()
        && let Ok(glyph) = icon::beacon(palette::GLYPH)
    {
        group.set_icon(&doc, Some(glyph));
    }

    let travel = Mote::new(Kind::Cast, "Travel").ok()?;
    travel.describe("Go to this space.").ok()?;
    travel.set_tint(Some(color));

    let beacon = Mote::new(Kind::Item, "Beacon").ok()?;
    beacon.describe("A marker you can drop here.").ok()?;
    beacon.set_tint(Some(color));
    // The beacon itself is a cube of corners around a pulsing core, so its
    // glyph is the same form. A missing glyph is not a reason to lose the
    // whole space.
    if let Ok(doc) = script_document()
        && let Ok(glyph) = icon::beacon(palette::GLYPH)
    {
        beacon.set_icon(&doc, Some(glyph));
    }

    group.add_child(&travel);
    group.add_child(&beacon);

    Some(Space {
        ns: listing.ns,
        hex: listing.hex.clone(),
        group,
        travel,
        beacon,
    })
}

/// Occupancy is a real fact about a real place, so a full space reads as full
/// and the halo never offers to make another copy of one.
fn describe(listing: &Listing) -> String {
    let people = match listing.occupants {
        1 => "1 person here".to_owned(),
        count => format!("{count} people here"),
    };
    if listing.idle_secs < 60 {
        return people;
    }
    format!("{people}, quiet for {} min", listing.idle_secs / 60)
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
    let prim = beacon.create_prim(Layer::Local, None)?;
    // The beacon script finds itself by a prim named for its space.
    beacon
        .local()
        .set(prim, Property::Name(hex.to_owned()))
        .set(prim, Property::Transform(Transform::from_translation(at)))
        .flush()?;
    // Planting runs on this peer alone, so the prim must be committed for
    // anyone else's copy of the beacon to find itself.
    let keys: Vec<_> = beacon
        .keys(prim)
        .into_iter()
        .map(|key| (prim, key))
        .collect();
    beacon.commit(&keys)?;
    beacon.place(Anchor::Space, Transform::IDENTITY)?;

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
