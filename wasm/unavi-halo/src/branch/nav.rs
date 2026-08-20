//! Nav: the spaces you can go to.
//!
//! A level that opens as a grid, so the listing gets pagination, attention and
//! placards for free.

use std::str::FromStr;

use blake3::Hash;
use unavi_script_util::color::generate_color;
use wired_prelude::prelude::*;

use crate::{
    icon,
    palette,
    unavi::vui::api::{
        Kind,
        Landing,
        Mote,
    },
    wired::{
        portal::api::travel,
        scene::{
            api::{
                create_document_from_prefab,
                self_document,
            },
            types::Document,
        },
        wds::{
            api::get_wds,
            types::ListFuture,
        },
    },
};

/// Prefix of a registry's active-spaces view; listing it picks out activity
/// and ignores the rest. The other views a registry publishes share the
/// namespace list and answer this prefix with nothing.
const ACTIVE_PREFIX: &str = "active/";

/// The authored prim whose `prefab` slot every beacon copies, kept at zero
/// scale so the template itself never shows.
const TEMPLATE_PRIM_NAME: &str = "beacon_template";

/// A space the registries say has people in it.
struct Space {
    /// The 32-byte namespace, for travelling.
    ns:     Vec<u8>,
    /// The same, as the registry wrote it, which is what a beacon is named.
    hex:    String,
    group:  Mote,
    travel: Mote,
    beacon: Mote,
}

/// Lists the spaces currently occupied per the registries this client follows.
#[derive(Default)]
pub struct Nav {
    lists:   Vec<ListFuture>,
    spaces:  Vec<Space>,
    beacons: Vec<Document>,
}

impl Nav {
    /// Asks every registry what is live. Called when the branch opens, so a
    /// halo that is never opened costs nothing.
    pub fn refresh(&mut self) {
        let Ok(wds) = get_wds() else {
            return;
        };
        self.lists = wds
            .registries()
            .iter()
            .map(|registry| wds.list(registry, ACTIVE_PREFIX))
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
                Err(()) => eprintln!("halo: registry active-space list error"),
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
        if let Err(err) = travel(&space.ns) {
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
    ns:        Vec<u8>,
    occupants: u32,
    idle_secs: u64,
}

/// Decodes an `active/<rank>/<space-hex>` entry.
///
/// The value carries occupancy. There is no display name in this view — that
/// lives in the registry's other views behind a payload a guest cannot decode
/// — so a space is named by the head of its namespace, and the placard says
/// only what is actually known.
fn entry(entry: &crate::wired::wds::types::Entry) -> Option<Listing> {
    let mut parts = entry.key.strip_prefix(ACTIVE_PREFIX)?.split('/');
    let rank = parts.next()?.parse().ok()?;
    let space = parts.next().filter(|space| !space.is_empty())?;
    let (occupants, idle_secs) = postcard::from_bytes(&entry.value).ok()?;

    Some(Listing {
        rank,
        ns: from_hex(space)?,
        hex: space.to_string(),
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

    let group = Mote::new(Kind::Group, listing.hex.get(..8)?);
    group.describe(&describe(listing));
    group.set_tint(Some(color));
    // The space wears its beacon's form: in the grid it reads as the marker
    // it is, and opening it still shows the travel and beacon motes beneath.
    group.set_icon(icon::beacon(palette::GLYPH).ok().as_ref());

    let travel = Mote::new(Kind::Cast, "Travel");
    travel.describe("Go to this space.");
    travel.set_tint(Some(color));

    let beacon = Mote::new(Kind::Item, "Beacon");
    beacon.describe("A marker you can drop here.");
    beacon.set_tint(Some(color));
    // The beacon itself is a cube of corners around a pulsing core, so its
    // glyph is the same form. A missing glyph is not a reason to lose the
    // whole space.
    beacon.set_icon(icon::beacon(palette::GLYPH).ok().as_ref());

    group.add_child(&travel);
    group.add_child(&beacon);

    Some(Space {
        ns: listing.ns.clone(),
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
        1 => "1 person here".to_string(),
        count => format!("{count} people here"),
    };
    if listing.idle_secs < 60 {
        return people;
    }
    format!("{people}, quiet for {} min", listing.idle_secs / 60)
}

fn mint(hex: &str, at: Vec3) -> anyhow::Result<Document> {
    let doc = self_document()?;
    let template = doc
        .prims()
        .into_iter()
        .find(|prim| prim.name().is_some_and(|name| name == TEMPLATE_PRIM_NAME))
        .and_then(|prim| prim.prefab())
        .ok_or_else(|| anyhow::anyhow!("halo HSD is missing its {TEMPLATE_PRIM_NAME} prim"))?;

    // Built in full while the document is still parked, so the room sees a
    // beacon appear where it was let go rather than one arriving at the origin
    // and moving.
    let beacon = create_document_from_prefab(&template)?;
    let prim = beacon.create_prim()?;
    // The beacon script finds itself by a prim named for its space.
    prim.set_name(Some(hex))?;
    prim.set_xform(Some(placed(at)))?;
    beacon.set_offset(placed(Vec3::ZERO))?;

    Ok(beacon)
}

const fn placed(translation: Vec3) -> Transform {
    Transform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }
}

/// The registry writes a namespace as hex; travelling wants the bytes.
fn from_hex(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    hex.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(digits, 16).ok()
        })
        .collect()
}
