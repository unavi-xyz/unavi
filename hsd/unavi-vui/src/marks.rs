//! VUI's own marks, drawn as dot glyphs like any other: the way back and the
//! page tabs.
//!
//! They are authored as rows of text, so each can be read and edited as the
//! picture it is, and packed once on the way in.

use std::rc::Rc;

use wired_guest::glyph::pack;

use crate::{
    glyph::{
        Glyph,
        Refused,
    },
    mote::Role,
};

/// The way back: a chevron pointing up.
pub const UP: [&str; 9] = [
    ".........",
    ".........",
    "....#....",
    "...###...",
    "..##.##..",
    ".##...##.",
    ".#.....#.",
    ".........",
    ".........",
];

/// A tab back a page: a chevron pointing left.
pub const LEFT: [&str; 9] = [
    ".........",
    ".....##..",
    "....##...",
    "...##....",
    "..##.....",
    "...##....",
    "....##...",
    ".....##..",
    ".........",
];

/// A tab forward a page: a chevron pointing right.
pub const RIGHT: [&str; 9] = [
    ".........",
    "..##.....",
    "...##....",
    "....##...",
    ".....##..",
    "....##...",
    "...##....",
    "..##.....",
    ".........",
];

/// `rows` as a glyph.
pub fn glyph(rows: &[&str]) -> Result<Glyph, Refused> {
    let (size, bits) = pack(rows);
    Glyph::new(size, &bits)
}

/// The chevrons the way back and the page tabs wear, built once.
pub struct Chevrons {
    up:    Rc<Glyph>,
    left:  Rc<Glyph>,
    right: Rc<Glyph>,
}

impl Chevrons {
    pub fn new() -> Result<Self, Refused> {
        Ok(Self {
            up:    Rc::new(glyph(&UP)?),
            left:  Rc::new(glyph(&LEFT)?),
            right: Rc::new(glyph(&RIGHT)?),
        })
    }

    /// The chevron a slot of `role` wears, for the roles that wear one. The
    /// way back points up and a tab points the way it turns.
    pub const fn of(&self, role: Role) -> Option<&Rc<Glyph>> {
        match role {
            Role::Parent { .. } => Some(&self.up),
            Role::Page { forward: false, .. } => Some(&self.left),
            Role::Page { forward: true, .. } => Some(&self.right),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use wired_guest::glyph::centred;

    use super::*;
    use crate::glyph::inside;

    const ALL: [(&str, &[&str]); 3] = [("UP", &UP), ("LEFT", &LEFT), ("RIGHT", &RIGHT)];

    #[test]
    fn every_mark_is_centred_in_its_grid() {
        for (name, rows) in ALL {
            assert!(centred(rows), "{name} slides to one side of its orb");
        }
    }

    #[test]
    fn every_dot_of_every_mark_is_inside_the_clip() {
        for (name, rows) in ALL {
            for (row, line) in rows.iter().enumerate() {
                for (column, cell) in line.bytes().enumerate() {
                    assert!(
                        cell != b'#' || inside(rows.len(), column, row),
                        "{name} has a dot at {column}, {row} that would be dropped"
                    );
                }
            }
        }
    }

    #[test]
    fn a_right_chevron_is_a_left_one_turned_about() {
        let turned: Vec<String> = LEFT.iter().map(|row| row.chars().rev().collect()).collect();
        assert_eq!(turned, RIGHT);
    }

    #[test]
    fn the_way_back_and_the_tabs_each_wear_a_chevron_and_nothing_else_does() {
        let chevrons = Chevrons::new().expect("chevrons");
        let up = chevrons.of(Role::Parent { depth: 1 }).expect("up");
        let left = chevrons
            .of(Role::Page {
                forward:   false,
                remaining: 1,
            })
            .expect("left");
        let right = chevrons
            .of(Role::Page {
                forward:   true,
                remaining: 1,
            })
            .expect("right");
        assert_ne!(up, left);
        assert_ne!(left, right);
        assert!(chevrons.of(Role::Action).is_none());
        assert!(chevrons.of(Role::Toggle).is_none());
    }
}
