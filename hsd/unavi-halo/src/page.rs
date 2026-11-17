//! What the page over the halo's root says while no key is attended: the
//! time, where you are, and who is there.

use crate::{
    branch::nav::Presence,
    unavi::vui::api::{
        Orbit,
        Resting,
    },
};

/// What a user in no space is told.
const BETWEEN: &str = "Between spaces";

/// What a user in a space whose listing is not known is told.
const HERE: &str = "You are here";

/// The root page's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rest {
    title: String,
    lines: Vec<String>,
}

impl Rest {
    /// The page for `time` and where the user is: the space's title over the
    /// time and who is in the space.
    #[must_use]
    pub fn of(time: Option<String>, presence: &Presence) -> Self {
        let title = if !presence.in_space {
            BETWEEN.to_owned()
        } else {
            presence
                .title
                .as_deref()
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .unwrap_or(HERE)
                .to_owned()
        };
        Self {
            title,
            lines: time.into_iter().chain(presence.people.clone()).collect(),
        }
    }
}

/// Keeps the orbit's resting page current, writing it only when it changes.
#[derive(Default)]
pub struct Page {
    shown: Option<Rest>,
}

impl Page {
    pub fn show(&mut self, orbit: &Orbit, rest: Rest) {
        if self.shown.as_ref() == Some(&rest) {
            return;
        }
        let page = Resting {
            title: rest.title.clone(),
            lines: rest.lines.clone(),
        };
        match orbit.set_resting(Some(&page)) {
            Ok(()) => self.shown = Some(rest),
            Err(err) => eprintln!("halo: resting page: {err:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presence(in_space: bool, title: Option<&str>, people: Option<&str>) -> Presence {
        Presence {
            in_space,
            title: title.map(str::to_owned),
            people: people.map(str::to_owned),
        }
    }

    #[test]
    fn a_space_is_its_title_over_the_time_and_its_people() {
        let rest = Rest::of(
            Some("14:07".to_owned()),
            &presence(true, Some("  Plaza "), Some("3 people")),
        );
        assert_eq!(rest.title, "Plaza");
        assert_eq!(rest.lines, ["14:07", "3 people"]);
    }

    #[test]
    fn an_unlisted_space_says_you_are_here() {
        for title in [None, Some(""), Some("   ")] {
            let rest = Rest::of(None, &presence(true, title, None));
            assert_eq!(rest.title, HERE);
        }
    }

    #[test]
    fn limbo_says_between_spaces_whatever_it_remembers() {
        let rest = Rest::of(None, &presence(false, Some("Plaza"), Some("3 people")));
        assert_eq!(rest.title, BETWEEN);
    }

    #[test]
    fn the_time_leads_and_a_missing_one_leaves_the_rest() {
        let rest = Rest::of(None, &presence(true, Some("Plaza"), Some("1 person")));
        assert_eq!(rest.lines, ["1 person"]);
    }
}
