//! Which lens motes a level should hold, given what the visor announced.

use std::collections::HashSet;

use unavi_lens::Lens;

/// The most lens motes shown: an orbit's capacity less its way back.
pub const LENS_MAX: usize = 8;

/// What to do to the motes already held to match an announcement.
#[derive(Debug, PartialEq, Eq)]
pub struct Plan<'a> {
    /// The lenses to show, in the order they are announced. Each is either
    /// already held, by id, or needs a mote built.
    pub show: Vec<&'a Lens>,
    /// Ids held that the announcement no longer shows.
    pub drop: Vec<String>,
}

/// Matches `have`, the ids held now, to `announced`.
///
/// A lens announced twice is shown once, at its first place, and lenses past
/// [`LENS_MAX`] are not shown, so a visor cannot make the level page.
#[must_use]
pub fn plan<'a>(have: &[&str], announced: &'a [Lens]) -> Plan<'a> {
    let mut seen = HashSet::new();
    let show: Vec<&Lens> = announced
        .iter()
        .filter(|lens| seen.insert(lens.id.as_str()))
        .take(LENS_MAX)
        .collect();
    let drop = have
        .iter()
        .filter(|id| !show.iter().any(|lens| lens.id == **id))
        .map(|id| (*id).to_owned())
        .collect();
    Plan { show, drop }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lens(id: &str) -> Lens {
        Lens {
            id:          id.to_owned(),
            name:        id.to_owned(),
            description: String::new(),
            worn:        false,
        }
    }

    fn ids<'a>(plan: &Plan<'a>) -> Vec<&'a str> {
        plan.show.iter().map(|lens| lens.id.as_str()).collect()
    }

    #[test]
    fn a_first_announcement_shows_everything_in_order() {
        let announced = [lens("a"), lens("b")];
        let plan = plan(&[], &announced);
        assert_eq!(ids(&plan), ["a", "b"]);
        assert!(plan.drop.is_empty());
    }

    #[test]
    fn a_new_lens_is_added_and_the_rest_stay() {
        let announced = [lens("a"), lens("b"), lens("c")];
        let plan = plan(&["a", "b"], &announced);
        assert_eq!(ids(&plan), ["a", "b", "c"]);
        assert!(plan.drop.is_empty());
    }

    #[test]
    fn a_reordering_follows_the_announcement() {
        let announced = [lens("b"), lens("a")];
        let plan = plan(&["a", "b"], &announced);
        assert_eq!(ids(&plan), ["b", "a"]);
        assert!(plan.drop.is_empty());
    }

    #[test]
    fn a_lens_the_visor_dropped_is_dropped() {
        let announced = [lens("b")];
        let plan = plan(&["a", "b"], &announced);
        assert_eq!(ids(&plan), ["b"]);
        assert_eq!(plan.drop, ["a"]);
    }

    #[test]
    fn an_empty_announcement_drops_everything() {
        let plan = plan(&["a"], &[]);
        assert!(plan.show.is_empty());
        assert_eq!(plan.drop, ["a"]);
    }

    #[test]
    fn a_repeated_id_is_shown_once() {
        let announced = [lens("a"), lens("b"), lens("a")];
        assert_eq!(ids(&plan(&[], &announced)), ["a", "b"]);
    }

    #[test]
    fn the_list_is_capped() {
        let announced: Vec<_> = (0..20).map(|n| lens(&n.to_string())).collect();
        let plan = plan(&["19"], &announced);
        assert_eq!(plan.show.len(), LENS_MAX);
        assert_eq!(plan.drop, ["19"]);
    }
}
