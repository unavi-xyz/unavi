//! The "This space" mote pinned at the head of Nav's grid.

use crate::{
    glyph,
    unavi::vui::api::{
        Kind,
        Mote,
    },
};

/// What the pinned mote says in a space whose listing is not known.
const HERE: &str = "You are here";

/// What it says between spaces.
const LIMBO: &str = "Between spaces";

/// The cast it wears in a space: travelling to the space you stand in drops
/// and re-reads it, which is the only way back from a space a script or a peer
/// has left broken.
const RESPAWN: (&str, &str) = ("Respawn", "Reload this space.");

/// The space the user is in, always first in the grid.
///
/// Listed spaces leave the grid while the user stands in them, so this is the
/// only entry for the current space and it never moves. The cast is attached
/// only in a space: the mote API has no way to hide a child, but it can take
/// one out and put it back, and a group with a single child has no order to
/// disturb.
pub(super) struct Here {
    pub(super) group: Mote,
    cast:             Mote,
    /// Whether the cast is attached to the group now.
    casting:          bool,
}

impl Here {
    pub(super) fn new() -> anyhow::Result<Self> {
        let group = Mote::new(Kind::Group, "This space")?;
        group.set_description(LIMBO)?;
        // A refused glyph is not a reason to lose the mote.
        if let Err(err) = group.set_glyph(Some(&glyph::from_rows(&glyph::HERE))) {
            eprintln!("halo: this space glyph: {err:?}");
        }

        let cast = Mote::new(Kind::Cast, RESPAWN.0)?;
        cast.set_description(RESPAWN.1)?;
        if let Err(err) = cast.set_glyph(Some(&glyph::from_rows(&glyph::TRAVEL))) {
            eprintln!("halo: respawn glyph: {err:?}");
        }

        Ok(Self {
            group,
            cast,
            casting: false,
        })
    }

    /// Rewrites the mote for where the user is. `title` is the current
    /// space's listing title when the registries have shown it.
    pub(super) fn show(&mut self, in_space: bool, title: Option<&str>) {
        let text = description(in_space, title);
        if let Err(err) = self.group.set_description(&text) {
            eprintln!("halo: this space description: {err:?}");
        }
        if in_space == self.casting {
            return;
        }
        self.casting = in_space;
        if in_space {
            self.group.add_child(&self.cast);
        } else {
            self.group.remove_child(&self.cast);
        }
    }

    pub(super) fn is_cast(&self, mote: &Mote) -> bool {
        self.cast.is(mote)
    }

    pub(super) fn notify(&self, message: &str) {
        self.cast.notify(message);
    }
}

/// The description for a user in a space with listing title `title`, or
/// between spaces.
fn description(in_space: bool, title: Option<&str>) -> String {
    if !in_space {
        return LIMBO.to_owned();
    }
    match title.map(str::trim) {
        Some(title) if !title.is_empty() => title.to_owned(),
        _ => HERE.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_known_title_is_the_description() {
        assert_eq!(description(true, Some("  Plaza ")), "Plaza");
    }

    #[test]
    fn a_blank_title_says_you_are_here() {
        assert_eq!(description(true, Some("")), HERE);
        assert_eq!(description(true, Some("   ")), HERE);
    }

    #[test]
    fn limbo_says_between_spaces() {
        assert_eq!(description(false, None), LIMBO);
        assert_eq!(description(false, Some("Plaza")), LIMBO);
        assert_eq!(description(true, None), HERE);
    }
}
