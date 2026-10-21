//! A short line in front of the viewer saying what the shell just did for
//! them, where nothing else in view changed to say it: an undo, a redo.

use wired_guest::math::{
    Transform,
    Vec3,
};

use crate::unavi::{
    author::api::{
        Step,
        Way,
    },
    vui::api::Placard,
};

/// Seconds a notice stays up.
const SHOWN: f32 = 1.6;
/// Metres ahead of the eye, and below it, a notice stands.
const AHEAD: f32 = 0.9;
const BELOW: f32 = 0.18;

pub struct Notice {
    placard: Placard,
    /// Seconds until it goes, zero while it is down.
    left:    f32,
}

impl Notice {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            placard: Placard::new()?,
            left:    0.0,
        })
    }

    /// Puts up what `step` did.
    pub fn step(&mut self, step: &Step) {
        let (done, none) = match step.way {
            Way::Undo => ("Undone", "Nothing to undo"),
            Way::Redo => ("Redone", "Nothing to redo"),
        };
        match &step.label {
            Some(label) => self.show(label, Some(done)),
            None => self.show(none, None),
        }
    }

    fn show(&mut self, title: &str, kicker: Option<&str>) {
        if let Err(err) = self.placard.set_text(title, kicker, &[]) {
            eprintln!("halo: notice: {err:?}");
            return;
        }
        self.placard.set_shown(true);
        self.left = SHOWN;
    }

    /// Keeps it in front of `eye` while it is up, and takes it down when its
    /// time runs out.
    pub fn update(&mut self, eye: &Transform, dt: f32) {
        if self.left <= 0.0 {
            return;
        }
        self.left -= dt;
        if self.left <= 0.0 {
            self.placard.set_shown(false);
            return;
        }
        let forward = eye.forward();
        let level = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let at = eye.translation + level * AHEAD - Vec3::Y * (BELOW + self.placard.size().y);
        self.placard.set_position(at);
    }
}
