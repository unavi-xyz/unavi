//! What a portal's tag says.

use unavi_lens::DocumentId;
use wired_guest::color::document_seed;

use crate::{
    rung,
    wired::perception::perception::TrustRung,
};

/// Characters of a title kept before it is cut with an ellipsis.
const TITLE_MAX: usize = 48;

/// Hex digits of the destination's id the kicker shows.
const ID_DIGITS: usize = 12;
/// Hex digits between the kicker's separators.
const ID_GROUP: usize = 4;

/// The title of a destination nothing names.
const UNNAMED: &str = "Unnamed space";

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

/// A tag's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    /// The head of the destination's id, over the title.
    pub kicker:  String,
    pub title:   String,
    /// Who is there and whose it is, in one line, once either is known.
    pub facts:   Option<String>,
    /// Whether there is no way back, said as a warning.
    pub one_way: bool,
}

/// Names the portal's destination, then says who is there, whose it is and
/// whether it comes back.
#[must_use]
pub fn describe(facts: &Facts) -> Content {
    let known: Vec<String> = [
        facts.occupants.map(here),
        facts.author.map(|rung| rung::word(rung).to_owned()),
    ]
    .into_iter()
    .flatten()
    .collect();
    Content {
        kicker:  kicker(facts.destination),
        title:   title(facts),
        facts:   (!known.is_empty()).then(|| known.join(" · ")),
        one_way: !facts.linked,
    }
}

/// The listing title, else the space's own name.
fn title(facts: &Facts) -> String {
    [facts.listed_title, facts.meta_name]
        .into_iter()
        .flatten()
        .map(str::trim)
        .find(|title| !title.is_empty())
        .map_or_else(|| UNNAMED.to_owned(), clip)
}

/// The id's head in upper-case groups, `A3F9·0C21·7BE4`.
fn kicker(destination: DocumentId) -> String {
    let hex = document_seed(destination).to_hex();
    let mut kicker = String::with_capacity(ID_DIGITS + ID_DIGITS / ID_GROUP * 2);
    for (index, digit) in hex.chars().take(ID_DIGITS).enumerate() {
        if index > 0 && index % ID_GROUP == 0 {
            kicker.push('·');
        }
        kicker.push(digit.to_ascii_uppercase());
    }
    kicker
}

fn clip(title: &str) -> String {
    if title.chars().count() <= TITLE_MAX {
        return title.to_owned();
    }
    let mut clipped: String = title.chars().take(TITLE_MAX - 1).collect();
    clipped.push('…');
    clipped
}

fn here(count: u32) -> String {
    match count {
        0 => "Empty".to_owned(),
        count => format!("{count} here"),
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
    fn the_kicker_is_the_head_of_the_id_in_groups() {
        let hex: String = document_seed(DEST)
            .to_hex()
            .chars()
            .take(ID_DIGITS)
            .collect::<String>()
            .to_ascii_uppercase();
        let expected = format!("{}·{}·{}", &hex[0..4], &hex[4..8], &hex[8..12]);
        assert_eq!(describe(&facts()).kicker, expected);
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
    fn what_is_known_reads_as_one_line() {
        let content = describe(&Facts {
            occupants: Some(3),
            author: Some(TrustRung::Trusted),
            ..facts()
        });
        assert_eq!(content.facts.as_deref(), Some("3 here · Trusted"));
    }

    #[test]
    fn an_empty_space_says_so() {
        let content = describe(&Facts {
            occupants: Some(0),
            ..facts()
        });
        assert_eq!(content.facts.as_deref(), Some("Empty"));
    }

    #[test]
    fn nothing_known_says_nothing() {
        let content = describe(&facts());
        assert_eq!(content.facts, None);
        assert!(!content.one_way);
    }

    #[test]
    fn a_one_way_portal_says_so() {
        let content = describe(&Facts {
            linked: false,
            ..facts()
        });
        assert!(content.one_way);
    }
}
