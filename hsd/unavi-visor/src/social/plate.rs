//! What a nameplate says.

use crate::{
    rung,
    wired::perception::perception::TrustRung,
};

/// Characters of a name kept before it is cut with an ellipsis.
const NAME_MAX: usize = 32;
/// Characters of a description kept before it is cut with an ellipsis.
const DESCRIPTION_MAX: usize = 96;

/// Characters of a title that fit one line of a standing placard, by VUI's
/// own estimate of its pane width over its title advance.
const TITLE_WIDTH: usize = 21;

/// Everything known about a user.
pub struct Facts<'a> {
    pub user:        &'a str,
    /// The name their profile gives, once it has arrived.
    pub name:        Option<&'a str>,
    pub description: Option<&'a str>,
    pub rung:        Option<TrustRung>,
}

/// A nameplate's text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plate {
    pub kicker: Option<String>,
    pub title:  String,
    /// Muted, under the title: the description of a named user.
    pub line:   Option<String>,
}

#[must_use]
pub fn describe(facts: &Facts) -> Plate {
    let name = facts.name.map(str::trim).filter(|name| !name.is_empty());
    let description = facts
        .description
        .map(str::trim)
        .filter(|description| !description.is_empty());
    Plate {
        kicker: facts.rung.map(|rung| rung::word(rung).to_owned()),
        title:  name.map_or_else(|| did_tail(facts.user), |name| clip(name, NAME_MAX)),
        line:   name
            .and(description)
            .map(|description| clip(description, DESCRIPTION_MAX)),
    }
}

/// `did` cut to the title's one line as its method, `did:key:…`, then as much
/// of its tail as fits. Every key of one type shares the head of its id, so
/// the tail is what tells two apart.
fn did_tail(did: &str) -> String {
    if did.chars().count() <= TITLE_WIDTH {
        return did.to_owned();
    }
    let Some((method, id)) = did.rsplit_once(':') else {
        return clip(did, TITLE_WIDTH);
    };
    let mut cut = format!("{method}:…");
    let keep = TITLE_WIDTH.saturating_sub(cut.chars().count());
    let count = id.chars().count();
    cut.extend(id.chars().skip(count.saturating_sub(keep)));
    cut
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut clipped: String = text.chars().take(max - 1).collect();
    clipped.push('…');
    clipped
}

#[cfg(test)]
mod tests {
    use super::*;

    const DID: &str = "did:key:zDnaeWgbpcUat3VPa1GxbBYnmJcmeYN6w1WJh9v1A2B3C4";

    fn facts() -> Facts<'static> {
        Facts {
            user:        DID,
            name:        None,
            description: None,
            rung:        None,
        }
    }

    #[test]
    fn a_named_user_is_titled_by_name() {
        let plate = describe(&Facts {
            name: Some(" Ada "),
            ..facts()
        });
        assert_eq!(plate.title, "Ada");
        assert_eq!(plate.line, None);
    }

    #[test]
    fn an_unnamed_user_is_titled_by_the_end_of_their_did() {
        let plate = describe(&facts());
        assert_eq!(plate.title, "did:key:…WJh9v1A2B3C4");
        assert_eq!(plate.title.chars().count(), TITLE_WIDTH);
        assert_eq!(plate.line, None, "nothing under an unnamed title");
    }

    #[test]
    fn a_short_did_is_shown_whole() {
        let plate = describe(&Facts {
            user: "did:web:a.io",
            ..facts()
        });
        assert_eq!(plate.title, "did:web:a.io");
    }

    #[test]
    fn a_blank_name_counts_as_none() {
        let plate = describe(&Facts {
            name: Some("   "),
            description: Some("ignored without a name"),
            ..facts()
        });
        assert_eq!(plate.title, "did:key:…WJh9v1A2B3C4");
        assert_eq!(plate.line, None);
    }

    #[test]
    fn the_description_stands_under_the_name() {
        let plate = describe(&Facts {
            name: Some("Ada"),
            description: Some("Builds engines"),
            ..facts()
        });
        assert_eq!(plate.line.as_deref(), Some("Builds engines"));
    }

    #[test]
    fn the_kicker_is_the_rung() {
        let plate = describe(&Facts {
            rung: Some(TrustRung::Trusted),
            ..facts()
        });
        assert_eq!(plate.kicker.as_deref(), Some("Trusted"));
    }

    #[test]
    fn a_long_description_is_clipped() {
        let long = "y".repeat(500);
        let plate = describe(&Facts {
            name: Some("Ada"),
            description: Some(&long),
            ..facts()
        });
        let line = plate.line.expect("a description line");
        assert_eq!(line.chars().count(), DESCRIPTION_MAX);
        assert!(line.ends_with('…'));
    }

    #[test]
    fn a_long_name_is_clipped() {
        let long = "x".repeat(100);
        let plate = describe(&Facts {
            name: Some(&long),
            ..facts()
        });
        assert_eq!(plate.title.chars().count(), NAME_MAX);
        assert!(plate.title.ends_with('…'));
    }
}
