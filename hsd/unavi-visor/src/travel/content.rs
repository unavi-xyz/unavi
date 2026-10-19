//! What a portal's placard says.

use unavi_lens::DocumentId;
use wired_guest::color::document_seed;

use crate::wired::perception::perception::TrustRung;

/// Characters of a title kept before it is cut with an ellipsis.
const TITLE_MAX: usize = 48;

/// Hex digits of the destination shown when nothing else names it.
const HEAD_LEN: usize = 8;

/// Everything known about the far side of a portal.
pub struct Facts<'a> {
    /// The title the registries list it under.
    pub listed_title: Option<&'a str>,
    /// The name the space gives itself, once it is loaded.
    pub meta_name:    Option<&'a str>,
    pub destination:  DocumentId,
    /// People in it, once the registries have been read.
    pub occupants:    Option<u32>,
    /// Whether the portal has a partner back.
    pub linked:       bool,
    /// How far the local user trusts whoever authored the destination.
    pub author:       Option<TrustRung>,
}

/// A placard's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    pub title: String,
    pub lines: Vec<String>,
}

/// Names the portal's destination, then says who is there, whether it comes
/// back, and whose it is.
#[must_use]
pub fn describe(facts: &Facts) -> Content {
    let mut lines = Vec::new();
    if let Some(occupants) = facts.occupants {
        lines.push(people(occupants));
    }
    if !facts.linked {
        lines.push("One way".to_owned());
    }
    if let Some(rung) = facts.author {
        lines.push(format!("{} space", rung_word(rung)));
    }
    Content {
        title: title(facts),
        lines,
    }
}

/// The listing title, else the space's own name, else the head of its id.
fn title(facts: &Facts) -> String {
    [facts.listed_title, facts.meta_name]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|title| !title.is_empty())
        .map_or_else(|| head(facts.destination), clip)
}

fn head(destination: DocumentId) -> String {
    document_seed(destination)
        .to_hex()
        .chars()
        .take(HEAD_LEN)
        .collect()
}

fn clip(title: &str) -> String {
    if title.chars().count() <= TITLE_MAX {
        return title.to_owned();
    }
    let mut clipped: String = title.chars().take(TITLE_MAX - 1).collect();
    clipped.push('…');
    clipped
}

fn people(count: u32) -> String {
    match count {
        0 => "Empty".to_owned(),
        1 => "1 person".to_owned(),
        count => format!("{count} people"),
    }
}

const fn rung_word(rung: TrustRung) -> &'static str {
    match rung {
        TrustRung::Blocked => "Blocked",
        TrustRung::Anonymous => "Anonymous",
        TrustRung::Guest => "Guest",
        TrustRung::Trusted => "Trusted",
        TrustRung::Myself => "Your own",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEST: DocumentId = (1, 2, 3, 4);

    fn facts() -> Facts<'static> {
        Facts {
            listed_title: None,
            meta_name:    None,
            destination:  DEST,
            occupants:    None,
            linked:       true,
            author:       None,
        }
    }

    #[test]
    fn the_listing_title_comes_first() {
        let content = describe(&Facts {
            listed_title: Some(" Plaza "),
            meta_name: Some("Other"),
            ..facts()
        });
        assert_eq!(content.title, "Plaza");
    }

    #[test]
    fn a_blank_listing_falls_to_the_space_name() {
        let content = describe(&Facts {
            listed_title: Some("  "),
            meta_name: Some("Other"),
            ..facts()
        });
        assert_eq!(content.title, "Other");
    }

    #[test]
    fn with_no_name_at_all_the_title_is_the_head_of_the_id() {
        let expected: String = document_seed(DEST).to_hex().chars().take(8).collect();
        assert_eq!(describe(&facts()).title, expected);
        assert_eq!(expected.len(), 8);
    }

    #[test]
    fn a_long_title_is_clipped() {
        let long = "x".repeat(100);
        let content = describe(&Facts {
            listed_title: Some(&long),
            ..facts()
        });
        assert_eq!(content.title.chars().count(), TITLE_MAX);
        assert!(content.title.ends_with('…'));
    }

    #[test]
    fn occupancy_reads_as_a_count() {
        for (count, line) in [(0, "Empty"), (1, "1 person"), (5, "5 people")] {
            let content = describe(&Facts {
                occupants: Some(count),
                ..facts()
            });
            assert_eq!(content.lines, [line]);
        }
    }

    #[test]
    fn unknown_occupancy_says_nothing() {
        assert!(describe(&facts()).lines.is_empty());
    }

    #[test]
    fn a_one_way_portal_says_so() {
        let content = describe(&Facts {
            linked: false,
            ..facts()
        });
        assert_eq!(content.lines, ["One way"]);
    }

    #[test]
    fn the_author_rung_is_named() {
        let content = describe(&Facts {
            author: Some(TrustRung::Guest),
            ..facts()
        });
        assert_eq!(content.lines, ["Guest space"]);
    }

    #[test]
    fn lines_keep_their_order() {
        let content = describe(&Facts {
            occupants: Some(2),
            linked: false,
            author: Some(TrustRung::Trusted),
            ..facts()
        });
        assert_eq!(content.lines, ["2 people", "One way", "Trusted space"]);
    }
}
