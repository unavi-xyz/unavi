/// How far a document's authority reaches.
///
/// Separate from the API permissions because the two answer different
/// questions: a permission says whether this code may call something at all,
/// while trust says whether the space boundary applies to it once it does.
/// Folding them together is what let a single `System` permission silently
/// mean "and skip the firewall and membership checks too".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Trust {
    /// Content that arrived by being in a space. The default, and the only
    /// tier a stranger's document ever gets.
    #[default]
    Untrusted,
    /// A space's own document, authored by whoever published the space.
    Space,
    /// The shell and the tools it ships with.
    System,
}

impl Trust {
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
