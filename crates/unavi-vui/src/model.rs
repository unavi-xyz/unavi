use crate::mote::MoteSpec;

/// What a surface is a view of.
///
/// Levels are addressed by index path so a model never hands out identifiers
/// of its own, and `children` is asked one level at a time so an unbounded
/// collection costs nothing until it is opened.
pub trait Model {
    /// The mote standing for the collection itself.
    fn root(&self) -> MoteSpec;

    /// Motes at `path`, empty for a path that leads nowhere.
    ///
    /// **Ordering must be stable.** A sigil is derived from position, so a
    /// level that reorders itself between frames silently rebinds whatever
    /// muscle memory a user has built.
    fn children(&self, path: &[usize]) -> Vec<MoteSpec>;

    /// Fires the leaf at `path`. Blooming a group is navigation and never
    /// reaches this.
    fn activate(&mut self, path: &[usize]);
}
