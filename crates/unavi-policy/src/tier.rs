/// Where a document came from, which decides whether the space boundary
/// applies to it.
///
/// Separate from API permissions (which answer whether this code may call
/// something at all) and from [`crate::trust::Trust`], which ranks *peers*
/// rather than documents: a document's tier is a property of where it was
/// loaded from, while the trust in its owner is the viewer's own opinion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// Content that arrived by being in a space. The default, and the only
    /// tier a stranger's document ever gets.
    #[default]
    Untrusted,
    /// A space's own document, authored by whoever published the space.
    Space,
    /// The shell and the tools it ships with.
    System,
}

impl Tier {
    /// Whether writes and spatial events from this document ignore space
    /// membership.
    ///
    /// Only the shell does: it is placed outside any space and still has to
    /// reach into whichever one the user is standing in.
    #[must_use]
    pub const fn crosses_space_boundaries(self) -> bool {
        matches!(self, Self::System)
    }
}
