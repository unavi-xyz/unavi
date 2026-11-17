//! The lenses the visor offers, and which are worn.
//!
//! A lens is one toggle layer. Any combination can be worn at once; with none
//! worn the visor is quiet.

use std::collections::HashSet;

use unavi_lens::{
    DOCUMENTS,
    DocumentId,
    Hold,
    Lens,
    SOCIAL,
    TRAVEL,
    Wear,
};

pub struct Registry {
    lenses:  Vec<Lens>,
    /// The documents holding each lens, by the lens id.
    holders: HashSet<(DocumentId, String)>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            lenses:  vec![
                Lens {
                    id:          TRAVEL.to_owned(),
                    name:        "Travel".to_owned(),
                    description: "Tags beside the portals you look at.".to_owned(),
                    worn:        true,
                },
                Lens {
                    id:          SOCIAL.to_owned(),
                    name:        "Social".to_owned(),
                    description: "Nameplates over the people you look at.".to_owned(),
                    worn:        true,
                },
                Lens {
                    id:          DOCUMENTS.to_owned(),
                    name:        "Documents".to_owned(),
                    description: "Corner brackets on nearby documents, in their authors' colours."
                        .to_owned(),
                    worn:        false,
                },
            ],
            holders: HashSet::new(),
        }
    }
}

impl Registry {
    /// Every lens, as announced.
    #[must_use]
    pub fn lenses(&self) -> &[Lens] {
        &self.lenses
    }

    #[must_use]
    pub fn is_worn(&self, id: &str) -> bool {
        self.lenses.iter().any(|lens| lens.id == id && lens.worn)
    }

    /// Whether a lens draws: the user wears it, or a document holds it.
    #[must_use]
    pub fn is_shown(&self, id: &str) -> bool {
        self.is_worn(id) || self.holders.iter().any(|(_, held)| held == id)
    }

    /// Records `sender` taking up or laying down a lens. What is worn is left
    /// alone. A lens that is not offered is ignored.
    pub fn hold(&mut self, sender: DocumentId, hold: &Hold) {
        if !self.lenses.iter().any(|lens| lens.id == hold.lens) {
            return;
        }
        let holder = (sender, hold.lens.clone());
        if hold.held {
            self.holders.insert(holder);
        } else {
            self.holders.remove(&holder);
        }
    }

    /// Lets go of the holds of documents `loaded` says are gone, since a tool
    /// that unloads cannot lay its lens down.
    pub fn release_unloaded(&mut self, loaded: impl Fn(&DocumentId) -> bool) {
        self.holders.retain(|(sender, _)| loaded(sender));
    }

    /// Puts a lens on or takes it off. Whether anything changed: a lens that
    /// is not offered, or is already as asked, changes nothing.
    pub fn wear(&mut self, request: &Wear) -> bool {
        let Some(lens) = self.lenses.iter_mut().find(|lens| lens.id == request.id) else {
            return false;
        };
        if lens.worn == request.worn {
            return false;
        }
        lens.worn = request.worn;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wear(id: &str, worn: bool) -> Wear {
        Wear {
            id: id.to_owned(),
            worn,
        }
    }

    #[test]
    fn travel_is_worn_by_default() {
        assert!(Registry::default().is_worn(TRAVEL));
    }

    #[test]
    fn taking_a_lens_off_and_on_changes_it_each_time() {
        let mut registry = Registry::default();
        assert!(registry.wear(&wear(TRAVEL, false)));
        assert!(!registry.is_worn(TRAVEL));
        assert!(registry.wear(&wear(TRAVEL, true)));
        assert!(registry.is_worn(TRAVEL));
    }

    #[test]
    fn asking_for_what_already_holds_changes_nothing() {
        let mut registry = Registry::default();
        assert!(!registry.wear(&wear(TRAVEL, true)));
    }

    #[test]
    fn documents_is_off_by_default() {
        let registry = Registry::default();
        assert!(!registry.is_worn(DOCUMENTS));
        assert!(!registry.is_shown(DOCUMENTS));
    }

    fn hold(lens: &str, held: bool) -> Hold {
        Hold {
            lens: lens.to_owned(),
            held,
        }
    }

    #[test]
    fn a_held_lens_shows_without_being_worn() {
        let mut registry = Registry::default();
        registry.hold((1, 0, 0, 0), &hold(DOCUMENTS, true));
        assert!(registry.is_shown(DOCUMENTS));
        assert!(!registry.is_worn(DOCUMENTS));
        registry.hold((1, 0, 0, 0), &hold(DOCUMENTS, false));
        assert!(!registry.is_shown(DOCUMENTS));
    }

    #[test]
    fn a_lens_stays_shown_until_every_holder_lets_go() {
        let mut registry = Registry::default();
        registry.hold((1, 0, 0, 0), &hold(DOCUMENTS, true));
        registry.hold((2, 0, 0, 0), &hold(DOCUMENTS, true));
        registry.hold((1, 0, 0, 0), &hold(DOCUMENTS, false));
        assert!(registry.is_shown(DOCUMENTS));
    }

    #[test]
    fn letting_go_leaves_the_users_own_choice() {
        let mut registry = Registry::default();
        assert!(registry.wear(&wear(DOCUMENTS, true)));
        registry.hold((1, 0, 0, 0), &hold(DOCUMENTS, true));
        registry.hold((1, 0, 0, 0), &hold(DOCUMENTS, false));
        assert!(registry.is_worn(DOCUMENTS));
        assert!(registry.is_shown(DOCUMENTS));
    }

    #[test]
    fn a_holder_that_unloads_stops_holding() {
        let mut registry = Registry::default();
        registry.hold((1, 0, 0, 0), &hold(DOCUMENTS, true));
        registry.release_unloaded(|_| false);
        assert!(!registry.is_shown(DOCUMENTS));
    }

    #[test]
    fn holding_a_lens_that_is_not_offered_does_nothing() {
        let mut registry = Registry::default();
        registry.hold((1, 0, 0, 0), &hold("nope", true));
        assert!(!registry.is_shown("nope"));
    }

    #[test]
    fn an_unknown_lens_changes_nothing() {
        let mut registry = Registry::default();
        assert!(!registry.wear(&wear("nope", true)));
        assert!(!registry.is_worn("nope"));
    }
}
