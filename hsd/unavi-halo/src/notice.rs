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
/// Metres from the eye to the middle of a notice.
const DISTANCE: f32 = 1.4;
/// Degrees below level that a notice's middle sits from the eye.
const DIP_DEGREES: f32 = 17.0;
/// A notice's size, as a multiple of a placard read across a room: its
/// title about 2 cm tall, a glance below the line of sight.
const SCALE: f32 = 0.4;

pub struct Notice {
    placard: Placard,
    /// Seconds until it goes, zero while it is down.
    left:    f32,
}

impl Notice {
    pub fn new() -> anyhow::Result<Self> {
        let placard = Placard::new()?;
        placard.set_scale(SCALE);
        Ok(Self { placard, left: 0.0 })
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
        self.placard
            .set_position(stand_at(eye, self.placard.size().y));
    }
}

/// Where the bottom centre of a notice `height` tall stands: its middle is
/// `DISTANCE` from the eye along the heading, dipped `DIP_DEGREES` below
/// level.
fn stand_at(eye: &Transform, height: f32) -> Vec3 {
    let forward = eye.forward();
    let level = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let dip = DIP_DEGREES.to_radians();
    let middle =
        eye.translation + level * (DISTANCE * dip.cos()) - Vec3::Y * (DISTANCE * dip.sin());
    middle - Vec3::Y * (height * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notice_stands_at_its_distance_and_dip_from_the_eye() {
        let eye = Transform::from_translation(Vec3::new(1.0, 1.6, -2.0));
        let height = 0.2;
        let middle = stand_at(&eye, height) + Vec3::Y * (height * 0.5);
        let offset = middle - eye.translation;
        assert!((offset.length() - DISTANCE).abs() < 1.0e-4);
        let dip = (-offset.y).atan2(Vec3::new(offset.x, 0.0, offset.z).length());
        assert!((dip.to_degrees() - DIP_DEGREES).abs() < 1.0e-3);
    }
}
