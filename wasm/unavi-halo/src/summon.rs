//! Where the halo stands, and when it goes away.
//!
//! It is placed at the summon point and world-locked rather than parented to
//! the viewer: a surface that follows you is one you cannot walk up to, and a
//! hand-parented ring jitters with tracking noise and cannot be aimed at by
//! the hand carrying it.

use wired_prelude::prelude::*;

/// Metres from the summon point, squared, past which the halo closes itself.
const CLOSE_MOVE_SQ: f32 = 0.09;

/// Metres of travel past which a menu press means "bring it here" rather than
/// "put it away". Under this the press is a dismissal, so the halo can always
/// be closed from where it was opened.
const RESUMMON_MOVE_SQ: f32 = 0.0225;

/// How far the viewer must have turned for the same reading. Stored as the
/// cosine of the half-angle between the two facings.
const RESUMMON_TURN_COS: f32 = 0.95;

/// What a press or a step should do to the halo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Summon,
    Dismiss,
    None,
}

/// Where the halo was called up, so walking away from it can close it.
#[derive(Default)]
pub struct Summon {
    at:      Option<Transform>,
    /// Menu is an edge, and the host reports it as a level.
    pressed: bool,
}

impl Summon {
    #[must_use]
    pub const fn is_up(&self) -> bool {
        self.at.is_some()
    }

    /// The menu button went down. Up and standing still puts the halo away;
    /// up and somewhere else brings it here, so repositioning is one press.
    pub fn press(&mut self, eye: &Transform) -> Command {
        if self.pressed {
            return Command::None;
        }
        self.pressed = true;

        match self.at {
            Some(at) if !moved(&at, eye) => {
                self.at = None;
                Command::Dismiss
            }
            _ => {
                self.at = Some(*eye);
                Command::Summon
            }
        }
    }

    pub const fn release(&mut self) {
        self.pressed = false;
    }

    /// Walking away closes it, so the halo never becomes something you have to
    /// tidy up after.
    pub fn step(&mut self, eye: &Transform) -> Command {
        let Some(at) = self.at else {
            return Command::None;
        };
        let delta = eye.translation - at.translation;
        if delta.dot(delta) <= CLOSE_MOVE_SQ {
            return Command::None;
        }
        self.at = None;
        Command::Dismiss
    }

    /// Planting something from the halo puts it away: attention has moved to
    /// the thing that was just placed.
    pub const fn taken(&mut self) -> Command {
        if self.at.is_none() {
            return Command::None;
        }
        self.at = None;
        Command::Dismiss
    }
}

/// Whether the viewer has moved or turned enough that a press means "bring it
/// here" rather than "put it away".
fn moved(at: &Transform, eye: &Transform) -> bool {
    let delta = eye.translation - at.translation;
    if delta.dot(delta) > RESUMMON_MOVE_SQ {
        return true;
    }
    let was = at.rotation * Vec3::new(0.0, 0.0, -1.0);
    let now = eye.rotation * Vec3::new(0.0, 0.0, -1.0);
    was.dot(now) < RESUMMON_TURN_COS
}
