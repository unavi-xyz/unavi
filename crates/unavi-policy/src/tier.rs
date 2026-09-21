/// Where a document came from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    #[default]
    Peer,
    Space,
    System,
}

impl Tier {
    /// Whether writes and spatial events from this document ignore space
    /// membership.
    #[must_use]
    pub const fn crosses_space_boundaries(self) -> bool {
        matches!(self, Self::System)
    }
}
