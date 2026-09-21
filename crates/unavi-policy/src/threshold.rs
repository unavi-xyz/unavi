use bevy::prelude::*;

use crate::trust::Trust;

/// How much trust a document requires of a writer.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Threshold {
    pub min_trust: Trust,
}

impl Threshold {
    #[must_use]
    pub const fn own_only() -> Self {
        Self {
            min_trust: Trust::Myself,
        }
    }

    /// Whether `trust` clears this document's bar.
    #[must_use]
    pub const fn cleared_by(self, trust: Trust) -> bool {
        trust.clears(self.min_trust)
    }
}
