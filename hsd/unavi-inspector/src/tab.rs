//! The slate's tabs: which of the inspected prim's faces it lists, and so
//! which single overlay stands on the prim.

/// What the slate lists, and what stands on the prim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    /// Handles to move, turn and scale the prim.
    #[default]
    Transform,
    /// A tether to its parent and a dot on each child.
    Tree,
    /// Everything stated on it. No overlay.
    Attributes,
}

impl Tab {
    pub const ALL: [Self; 3] = [Self::Transform, Self::Tree, Self::Attributes];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Transform => "Transform",
            Self::Tree => "Tree",
            Self::Attributes => "Attributes",
        }
    }

    #[must_use]
    pub fn index(self) -> u32 {
        Self::ALL
            .iter()
            .position(|tab| *tab == self)
            .map_or(0, |index| index as u32)
    }

    #[must_use]
    pub fn at(index: u32) -> Option<Self> {
        Self::ALL.get(index as usize).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tab_is_found_again_by_its_index() {
        for tab in Tab::ALL {
            assert_eq!(Tab::at(tab.index()), Some(tab));
        }
        assert_eq!(Tab::at(Tab::ALL.len() as u32), None);
    }

    #[test]
    fn the_first_tab_is_the_default() {
        assert_eq!(Tab::default(), Tab::ALL[0]);
    }

    #[test]
    fn the_labels_fit_a_tabs_row() {
        assert!(Tab::ALL.len() <= 4);
        assert!(Tab::ALL.iter().all(|tab| tab.label().len() <= 16));
    }
}
