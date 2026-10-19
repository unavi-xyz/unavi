//! What the registries say about the spaces that have people in them.
//!
//! The same view the halo's nav lists, read here for the title and head count
//! of a portal's destination. A listing is only as fresh as the last pass, and
//! a space the registries do not list is one nobody is in.

use std::collections::HashMap;

use unavi_lens::DocumentId;

use crate::unavi::host::node_storage::{
    self,
    Entry,
    PendingEntries,
};

/// Prefix of a registry's active-spaces view; listing it picks out activity
/// and ignores the rest. The other views a registry publishes share the
/// namespace list and answer this prefix with nothing.
const ACTIVE_PREFIX: &str = "active/";

/// The documented bound on `list-entries`.
const LIST_LIMIT: u32 = 256;

/// About two seconds of fixed ticks between passes.
const REFRESH_TICKS: u32 = 128;

/// One space as a registry reported it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    pub occupants: u32,
    /// As the registry wrote it; may be blank.
    pub title:     String,
}

/// The registries' active spaces, re-read every few seconds.
#[derive(Default)]
pub struct Listings {
    pending:    Vec<PendingEntries>,
    /// What the pass in flight has found so far.
    building:   HashMap<DocumentId, Listing>,
    /// The last complete pass, or none before the first one lands.
    settled:    Option<HashMap<DocumentId, Listing>>,
    refresh_in: u32,
}

impl Listings {
    /// Collects what the registries have answered and asks again when it is
    /// time.
    pub fn fixed_update(&mut self) {
        for index in (0..self.pending.len()).rev() {
            let Some(result) = self.pending[index].poll() else {
                continue;
            };
            self.pending.remove(index);
            match result {
                Ok(entries) => {
                    for (ns, listing) in entries.iter().filter_map(entry) {
                        self.building.insert(ns, listing);
                    }
                }
                Err(err) => eprintln!("visor: registry list error: {err:?}"),
            }
            if self.pending.is_empty() {
                self.settled = Some(std::mem::take(&mut self.building));
            }
        }

        if !self.pending.is_empty() {
            return;
        }
        if self.refresh_in > 0 {
            self.refresh_in -= 1;
            return;
        }
        self.refresh_in = REFRESH_TICKS;
        self.refresh();
    }

    fn refresh(&mut self) {
        let registries = match node_storage::registries() {
            Ok(registries) => registries,
            Err(err) => {
                eprintln!("visor: registries: {err:?}");
                return;
            }
        };
        self.pending = registries
            .into_iter()
            .filter_map(|registry| {
                node_storage::list_entries(registry, ACTIVE_PREFIX, LIST_LIMIT)
                    .inspect_err(|err| eprintln!("visor: list_entries: {err:?}"))
                    .ok()
            })
            .collect();
        if self.pending.is_empty() {
            self.settled = Some(HashMap::new());
        }
    }

    /// The title a registry gave `ns`, if it lists it and the title is not
    /// blank.
    #[must_use]
    pub fn title(&self, ns: DocumentId) -> Option<&str> {
        let title = self.settled.as_ref()?.get(&ns)?.title.trim();
        (!title.is_empty()).then_some(title)
    }

    /// How many people the registries count in `ns`: none when they do not
    /// list it, and unknown before the first pass lands.
    #[must_use]
    pub fn occupants(&self, ns: DocumentId) -> Option<u32> {
        let settled = self.settled.as_ref()?;
        Some(settled.get(&ns).map_or(0, |listing| listing.occupants))
    }
}

/// Decodes an `active/<rank>/<space-hex>` entry.
///
/// The value carries occupancy and the listing title, which is all the view
/// holds: the rest of a listing lives in the registry's other views behind a
/// payload a guest cannot decode.
fn entry(entry: &Entry) -> Option<(DocumentId, Listing)> {
    let mut parts = entry.key.strip_prefix(ACTIVE_PREFIX)?.split('/');
    let _rank: u32 = parts.next()?.parse().ok()?;
    let space = parts.next().filter(|space| !space.is_empty())?;
    let (occupants, _idle_secs, title): (u32, u64, String) =
        postcard::from_bytes(&entry.value).ok()?;

    Some((document_id_from_hex(space)?, Listing { occupants, title }))
}

/// A namespace from the 64 hex digits a registry writes, as the 32 bytes read
/// four little-endian words at a time.
fn document_id_from_hex(hex: &str) -> Option<DocumentId> {
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
    use wired_guest::color::document_seed;

    use super::*;

    fn active(key: &str, occupants: u32, title: &str) -> Entry {
        Entry {
            key:   key.to_owned(),
            value: postcard::to_stdvec(&(occupants, 5_u64, title)).expect("encodes"),
        }
    }

    #[test]
    fn a_namespace_survives_its_hex() {
        let ns = (1, u64::MAX, 0x0123_4567_89AB_CDEF, 42);
        let hex = document_seed(ns).to_hex().to_string();
        assert_eq!(document_id_from_hex(&hex), Some(ns));
    }

    #[test]
    fn an_active_entry_decodes_to_its_space_and_listing() {
        let hex = "ab".repeat(32);
        let (ns, listing) =
            entry(&active(&format!("active/7/{hex}"), 3, "Plaza")).expect("decodes");
        assert_eq!(
            ns,
            (
                0xABAB_ABAB_ABAB_ABAB,
                0xABAB_ABAB_ABAB_ABAB,
                0xABAB_ABAB_ABAB_ABAB,
                0xABAB_ABAB_ABAB_ABAB
            )
        );
        assert_eq!(
            listing,
            Listing {
                occupants: 3,
                title:     "Plaza".to_owned(),
            }
        );
    }

    #[test]
    fn other_views_and_malformed_entries_are_skipped() {
        let hex = "ab".repeat(32);
        assert!(entry(&active(&format!("recent/7/{hex}"), 1, "")).is_none());
        assert!(entry(&active("active/x/ab", 1, "")).is_none());
        assert!(entry(&active("active/7/", 1, "")).is_none());
        assert!(entry(&active("active/7/abcd", 1, "")).is_none());
        assert!(
            entry(&Entry {
                key:   format!("active/7/{hex}"),
                value: vec![0xFF],
            })
            .is_none()
        );
    }

    #[test]
    fn before_the_first_pass_nothing_is_known() {
        let listings = Listings::default();
        assert_eq!(listings.occupants((1, 2, 3, 4)), None);
        assert_eq!(listings.title((1, 2, 3, 4)), None);
    }

    #[test]
    fn after_a_pass_an_unlisted_space_is_empty_and_a_blank_title_is_none() {
        let mut listings = Listings::default();
        listings.settled = Some(HashMap::from([(
            (1, 1, 1, 1),
            Listing {
                occupants: 2,
                title:     "  ".to_owned(),
            },
        )]));
        assert_eq!(listings.occupants((1, 1, 1, 1)), Some(2));
        assert_eq!(listings.title((1, 1, 1, 1)), None);
        assert_eq!(listings.occupants((9, 9, 9, 9)), Some(0));
    }
}
