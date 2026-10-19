//! Nav: the spaces you can go to.
//!
//! A level that opens as a grid, so the listing gets pagination, attention and
//! placards for free.
//!
//! The first entry is always "This space", standing for the space the user is
//! in. The listed spaces follow, minus the one the user is in.

mod here;

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
        Color,
        Transform,
        Vec3,
    },
    xform::hidden,
};

use self::here::Here;
use crate::{
    glyph,
    unavi::{
        host::node_storage::{
            self,
            Entry,
            PendingEntries,
        },
        shapes::api::Cuboid,
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
                Material,
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

/// About two seconds of fixed ticks between attempts at a beacon icon that
/// failed to build.
const ICON_RETRY_TICKS: u32 = 128;

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
    /// The registry's ordering key, which sorts the grid.
    rank:   u32,
    /// For travelling, and the beacon's own colour seed.
    ns:     Namespace,
    /// The same, as the registry wrote it, which is what a beacon is named.
    hex:    String,
    /// The listing title as the registry wrote it; may be blank.
    title:  String,
    group:  Mote,
    cast:   Mote,
    beacon: Mote,
}

/// A space's namespace as a `document-id`.
type Namespace = (u64, u64, u64, u64);

/// The cast a space wears.
const TRAVEL: (&str, &str) = ("Travel", "Travel to this space.");

/// Lists the spaces currently occupied per the registries this app follows.
#[derive(Default)]
pub struct Nav {
    lists:         Vec<PendingEntries>,
    spaces:        Vec<Space>,
    beacons:       Vec<Document>,
    /// The space the user was last seen in.
    here:          Option<Namespace>,
    /// The "This space" mote, built on the first update so it precedes every
    /// listing in the grid.
    pinned:        Option<Here>,
    /// The beacon body, built once and worn by every space's beacon mote.
    icon:          Option<(u64, u64)>,
    /// Ticks left before a failed icon build is tried again.
    icon_retry_in: u32,
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

    /// Hangs a mote under `parent` for each space that has appeared, after the
    /// pinned "This space" mote.
    ///
    /// Sorted by the registry's own rank and then by namespace, so the order
    /// is the same every frame however the listings interleave — slot order is
    /// position, and position is muscle memory. The space the user is in is
    /// left out, since the pinned mote stands for it.
    pub fn fixed_update(&mut self, parent: &Mote) {
        let mut dirty = self.pin(parent);
        dirty |= self.follow_user();
        if self.icon.is_none() {
            if self.icon_retry_in == 0 {
                match beacon_icon() {
                    Ok(icon) => self.icon = Some(icon),
                    Err(err) => {
                        eprintln!("halo: beacon icon: {err:?}");
                        self.icon_retry_in = ICON_RETRY_TICKS;
                    }
                }
            } else {
                self.icon_retry_in -= 1;
            }
        }

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
        for listing in found {
            if self.spaces.iter().any(|space| space.hex == listing.hex) {
                continue;
            }
            let Some(space) = build(&listing, self.icon) else {
                continue;
            };
            self.spaces.push(space);
            dirty = true;
        }

        if dirty {
            self.spaces
                .sort_by(|a, b| a.rank.cmp(&b.rank).then_with(|| a.hex.cmp(&b.hex)));
            self.sync(parent);
        }
    }

    /// Builds the pinned mote and hangs it under `parent`, which it must do
    /// before any listing so it is the grid's first child. Whether it was
    /// built just now.
    fn pin(&mut self, parent: &Mote) -> bool {
        if self.pinned.is_some() {
            return false;
        }
        match Here::new() {
            Ok(pinned) => {
                parent.add_child(&pinned.group);
                self.pinned = Some(pinned);
                true
            }
            Err(err) => {
                eprintln!("halo: this space mote: {err:?}");
                false
            }
        }
    }

    /// Rehangs the listed spaces under `parent` and rewrites the pinned mote.
    ///
    /// The mote API can take a child out and append it, but not insert at an
    /// index, so keeping the order means taking every listed space out and
    /// putting back those that are shown. The pinned mote is never touched and
    /// so stays first.
    fn sync(&mut self, parent: &Mote) {
        for space in &self.spaces {
            parent.remove_child(&space.group);
        }
        let namespaces: Vec<_> = self.spaces.iter().map(|space| space.ns).collect();
        for index in listed(&namespaces, self.here) {
            parent.add_child(&self.spaces[index].group);
        }
        let title = self
            .here
            .and_then(|ns| self.spaces.iter().find(|space| space.ns == ns))
            .map(|space| space.title.as_str());
        if let Some(pinned) = &mut self.pinned {
            pinned.show(self.here.is_some(), title);
        }
    }

    /// Follows the space the user is in. A user between spaces, or a host that
    /// will not say, is in none. Whether that changed.
    fn follow_user(&mut self) -> bool {
        let here = current_space().ok().flatten();
        if here == self.here {
            return false;
        }
        self.here = here;
        // Beacons belong to the space they were planted in, and travel
        // despawns it, so the handles would only pile up pointing at nothing.
        self.beacons.clear();
        true
    }

    /// Travels to whichever space's cast just filled.
    pub fn cast(&self, mote: &Mote) -> bool {
        if let Some(pinned) = &self.pinned
            && pinned.is_cast(mote)
        {
            self.respawn(pinned);
            return true;
        }
        let Some(space) = self.spaces.iter().find(|space| space.cast.is(mote)) else {
            return false;
        };
        if let Err(err) = travel(space.ns) {
            eprintln!("halo: travel failed: {err:?}");
            space.cast.notify("Couldn't travel there.");
        }
        true
    }

    /// Travels to the space the user is already in, which drops and re-reads
    /// it. The cast is only attached in a space, so the notice for limbo is
    /// a guard against a fill landing as the user leaves.
    fn respawn(&self, pinned: &Here) {
        let Some(ns) = self.here else {
            pinned.notify("Not in a space");
            return;
        };
        if let Err(err) = travel(ns) {
            eprintln!("halo: respawn failed: {err:?}");
            pinned.notify("Couldn't reload this space.");
        }
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
    ns:        Namespace,
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
/// Two motes rather than one because the roles say different things: the cast
/// is consequential and gets a fill ring, and only an item leaves its slot to
/// be thrown. A mote cannot be both.
///
/// Every space wears the colour its beacon will: the beacon's hue is derived
/// from the space's own id, so the mote you pick out of the halo and the beacon
/// you put down in the world are the same colour, and a grid of spaces
/// separates at a glance rather than reading as one green sheet.
fn build(listing: &Listing, icon: Option<(u64, u64)>) -> Option<Space> {
    let color = generate_color(Hash::from_str(&listing.hex).ok()?);

    let group = Mote::new(Kind::Group, &label(listing)?).ok()?;
    let people = people(listing);
    group.set_description(&people).ok()?;
    group.set_tint(Some(color));
    // The space wears its beacon's mark: in the grid it reads as the marker
    // it is, and opening it still shows the cast and beacon motes beneath.
    // A refused glyph is not a reason to lose the whole space.
    if let Err(err) = group.set_glyph(Some(&glyph::from_rows(&glyph::BEACON))) {
        eprintln!("halo: space glyph: {err:?}");
    }

    let cast = Mote::new(Kind::Cast, TRAVEL.0).ok()?;
    cast.set_description(TRAVEL.1).ok()?;
    cast.set_tint(Some(color));
    // One mark for travel and respawn alike: both are the same act, and the
    // label says which. A refused glyph is not a reason to lose the space.
    if let Err(err) = cast.set_glyph(Some(&glyph::from_rows(&glyph::TRAVEL))) {
        eprintln!("halo: travel glyph: {err:?}");
    }

    let beacon = Mote::new(Kind::Item, "Beacon").ok()?;
    beacon
        .set_description("Drop a beacon to find your way back.")
        .ok()?;
    beacon.set_tint(Some(color));
    // The beacon body rather than a glyph: a mote that is carried should read
    // as the thing it is about to become.
    if let Some(icon) = icon
        && let Ok(doc) = script_document()
    {
        beacon.set_icon(&doc, Some(icon));
    }

    group.add_child(&cast);
    group.add_child(&beacon);

    Some(Space {
        rank: listing.rank,
        ns: listing.ns,
        hex: listing.hex.clone(),
        title: listing.title.clone(),
        group,
        cast,
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

/// Who is in a space, and how long it has been quiet once that is worth
/// saying. Only occupancy: the title is already the label.
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

/// The beacon body as an icon: the corner cubes and core the planted beacon
/// is built from, so a carried beacon mote reads as the thing it will become.
/// Neutral, because the shell carries the space's colour.
///
/// One icon is shared by every space's beacon mote. Only one space's level is
/// open at a time, so only one beacon mote is ever drawn and the surface's
/// icon dock never has two slots claiming the same prim.
///
/// A build that fails partway removes every prim it made, so retries never
/// pile orphans into the halo's prim quota.
fn beacon_icon() -> anyhow::Result<(u64, u64)> {
    let doc = script_document()?;
    let mut made = Vec::new();
    let built = build_beacon_icon(&doc, &mut made);
    if built.is_err()
        && let Err(err) = made
            .iter()
            .fold(doc.local(), |batch, prim| batch.remove(*prim))
            .flush()
    {
        eprintln!("halo: removing a half-built beacon icon: {err:?}");
    }
    built
}

fn build_beacon_icon(doc: &Document, made: &mut Vec<(u64, u64)>) -> anyhow::Result<(u64, u64)> {
    const SIZE: f32 = 1.0;
    const CORNER: f32 = SIZE * 0.44;
    const CORNER_OFFSET: f32 = (SIZE - CORNER) * 0.5;
    const CORE: f32 = SIZE * 0.84;

    const SHELL: Color = Color {
        r: 0.82,
        g: 0.85,
        b: 0.90,
        a: 1.0,
    };
    const GLOW: Color = Color {
        r: 0.95,
        g: 0.97,
        b: 1.0,
        a: 1.0,
    };

    let root = doc.create_prim(Layer::Local, None)?;
    made.push(root);
    // Hidden until a beacon mote wears it, so the icon never stands at the
    // document's origin at its authored size.
    let mut batch = doc.local().set(root, Property::Transform(hidden()));
    let shell = Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(SHELL),
        double_sided: None,
        emissive:     None,
        metallic:     Some(0.3),
        roughness:    Some(0.7),
    };
    let core = Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(GLOW),
        double_sided: None,
        emissive:     Some(Color {
            r: GLOW.r * 0.5,
            g: GLOW.g * 0.5,
            b: GLOW.b * 0.5,
            a: 1.0,
        }),
        metallic:     Some(0.3),
        roughness:    Some(0.7),
    };
    for x in [-1.0_f32, 1.0] {
        for y in [-1.0_f32, 1.0] {
            for z in [-1.0_f32, 1.0] {
                let corner = Cuboid::new(Vec3::splat(CORNER)).mesh()?;
                made.push(corner);
                batch = batch
                    .set(corner, Property::Parent(Some(root)))
                    .set(
                        corner,
                        Property::Transform(Transform::from_translation(
                            Vec3::new(x, y, z) * CORNER_OFFSET,
                        )),
                    )
                    .set(corner, Property::Material(shell));
            }
        }
    }
    let middle = Cuboid::new(Vec3::splat(CORE)).mesh()?;
    made.push(middle);
    batch = batch
        .set(middle, Property::Parent(Some(root)))
        .set(middle, Property::Material(core));
    batch.flush()?;
    Ok(root)
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
    // Minting has no document to parent to while in limbo, and says so as an
    // error that `plant` turns into a notice on the mote.
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

/// The indices of the spaces to show, in their given order, leaving out the one
/// the user is in.
fn listed(namespaces: &[Namespace], here: Option<Namespace>) -> Vec<usize> {
    namespaces
        .iter()
        .enumerate()
        .filter(|(_, ns)| here != Some(**ns))
        .map(|(index, _)| index)
        .collect()
}

/// The registry writes a namespace as hex; a `document-id` is its bytes as
/// four little-endian words.
fn document_id_from_hex(hex: &str) -> Option<Namespace> {
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
    fn the_current_space_is_left_out_and_the_rest_keep_their_order() {
        let spaces = [(1, 0, 0, 0), (2, 0, 0, 0), (3, 0, 0, 0)];
        assert_eq!(listed(&spaces, Some((2, 0, 0, 0))), vec![0, 2]);
        assert_eq!(listed(&spaces, Some((1, 0, 0, 0))), vec![1, 2]);
    }

    #[test]
    fn the_left_space_comes_back_in_its_own_slot() {
        let spaces = [(1, 0, 0, 0), (2, 0, 0, 0), (3, 0, 0, 0)];
        assert_eq!(listed(&spaces, None), vec![0, 1, 2]);
        assert_eq!(listed(&spaces, Some((9, 0, 0, 0))), vec![0, 1, 2]);
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
