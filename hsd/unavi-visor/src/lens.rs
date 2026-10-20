//! The lenses the visor offers, and which are worn.
//!
//! A lens is one toggle layer. Any combination can be worn at once; with none
//! worn the visor is quiet.

use unavi_lens::{
    Lens,
    SOCIAL,
    TRAVEL,
    Wear,
};

pub struct Registry {
    lenses: Vec<Lens>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            lenses: vec![
                Lens {
                    id:          TRAVEL.to_owned(),
                    name:        "Travel".to_owned(),
                    description: "Placards beside the portals you look at.".to_owned(),
                    worn:        true,
                },
                Lens {
                    id:          SOCIAL.to_owned(),
                    name:        "Social".to_owned(),
                    description: "Nameplates over the people you look at.".to_owned(),
                    worn:        true,
                },
            ],
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
    fn an_unknown_lens_changes_nothing() {
        let mut registry = Registry::default();
        assert!(!registry.wear(&wear("nope", true)));
        assert!(!registry.is_worn("nope"));
    }
}
