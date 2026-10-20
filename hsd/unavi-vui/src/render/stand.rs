//! Placards a script stands where it likes, rather than on a mote.

use std::f32::consts::{
    PI,
    TAU,
};

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    palette::Palette,
    placard::{
        self,
        Placard,
        PlacardView,
    },
    render::{
        draw,
        placard::PlacardPrims,
        pop,
        shadow::Shadow,
        tether::TetherPrims,
        viewer,
    },
    tether::{
        self,
        Pane,
    },
    tuning::Tuning,
    wired::scene::{
        document::{
            Document,
            Layer,
        },
        properties::Property,
    },
};

/// A standing placard [`Stands`] is drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StandId(usize);

/// Metres and radians a drawn pose must be from its target to snap onto it,
/// so a settled placard stops writing its pose.
const SETTLE_DISTANCE: f32 = 5.0e-4;
const SETTLE_TURN: f32 = 5.0e-4;
/// A target further than this, or turned this much, is a different place
/// rather than a move: a slot handed to another body appears there instead of
/// flying across the view.
const SNAP_DISTANCE: f32 = 0.4;
const SNAP_TURN: f32 = PI / 4.0;

/// One placard: a root prim that carries its position and turn, and the prims
/// that draw it.
struct Standing {
    root:   (u64, u64),
    prims:  PlacardPrims,
    tether: TetherPrims,
    view:   Option<PlacardView>,
    at:     Vec3,
    /// Colours the pane's corner tab and the tether.
    accent: Option<Color>,
    /// Where the tether ends, in the document frame.
    to:     Option<Vec3>,
    shown:  bool,
    /// The pose last drawn, eased toward the one the script puts it at.
    drawn:  Option<Transform>,
    pose:   Shadow<Transform>,
}

/// Every standing placard a script holds, as a free list like the panels.
#[derive(Default)]
pub struct Stands {
    slots: Vec<Option<Standing>>,
    free:  Vec<usize>,
}

impl Stands {
    pub fn add(&mut self, doc: &Document) -> anyhow::Result<StandId> {
        let root = doc.create_prim(Layer::Local, None)?;
        doc.local()
            .set(root, Property::Transform(draw::hidden()))
            .flush()?;
        let standing = Standing {
            root,
            prims: PlacardPrims::new(doc, root)?,
            tether: TetherPrims::new(doc)?,
            view: None,
            at: Vec3::ZERO,
            accent: None,
            to: None,
            shown: false,
            drawn: None,
            pose: Shadow::new(),
        };
        if let Some(index) = self.free.pop() {
            self.slots[index] = Some(standing);
            return Ok(StandId(index));
        }
        self.slots.push(Some(standing));
        Ok(StandId(self.slots.len() - 1))
    }

    pub fn remove(&mut self, doc: &Document, id: StandId) -> anyhow::Result<()> {
        let Some(standing) = self.slots.get_mut(id.0).and_then(Option::take) else {
            return Ok(());
        };
        self.free.push(id.0);
        // Both go whichever fails, so a failed root does not strand the
        // tether.
        let root = doc.local().remove(standing.root).flush();
        let tether = standing.tether.remove(doc);
        root?;
        tether
    }

    pub fn set_text(&mut self, id: StandId, text: &Placard, tuning: &Tuning) {
        if let Some(standing) = self.get_mut(id) {
            let mut view = placard::standing(text, tuning);
            view.accent = standing.accent;
            standing.view = Some(view);
        }
    }

    /// The pane as it will stand, or nothing before it has text.
    #[must_use]
    pub fn size(&self, id: StandId) -> Vec2 {
        self.slots
            .get(id.0)
            .and_then(Option::as_ref)
            .and_then(|standing| standing.view.as_ref())
            .map_or(Vec2::ZERO, |view| view.size)
    }

    pub fn set_position(&mut self, id: StandId, at: Vec3) {
        if let Some(standing) = self.get_mut(id) {
            standing.at = at;
        }
    }

    pub fn set_accent(&mut self, id: StandId, accent: Option<Color>) {
        if let Some(standing) = self.get_mut(id) {
            standing.accent = accent;
            if let Some(view) = &mut standing.view {
                view.accent = accent;
            }
        }
    }

    pub fn set_tether(&mut self, id: StandId, to: Option<Vec3>) {
        if let Some(standing) = self.get_mut(id) {
            standing.to = to;
        }
    }

    pub fn set_shown(&mut self, id: StandId, shown: bool) {
        if let Some(standing) = self.get_mut(id) {
            standing.shown = shown;
        }
    }

    /// Turns each placard toward `eye`, eases it toward where its script puts
    /// it, steps its pop, and lays its tether from wherever the pane has got
    /// to.
    pub fn update(
        &mut self,
        doc: &Document,
        eye: &Transform,
        palette: &Palette,
        tuning: &Tuning,
        delta: f32,
    ) -> anyhow::Result<()> {
        for standing in self.slots.iter_mut().flatten() {
            let rotation = turned_toward(standing.at, eye);
            let target = Transform {
                translation: standing.at,
                rotation,
                scale: Vec3::ONE,
            };
            if standing.shown && standing.view.is_some() {
                let drawn = eased(standing.drawn, target, tuning.stand_speed, delta);
                standing.drawn = Some(drawn);
                if let Some(pose) = standing.pose.diff(drawn) {
                    doc.local()
                        .set(standing.root, Property::Transform(pose))
                        .flush()?;
                }
            }
            standing.prims.update(
                doc,
                standing.view.as_ref().filter(|_| standing.shown),
                palette,
                delta,
            )?;

            // The tether pops with the pane: it reaches the pane's edge as the
            // pane grows from its centre, and goes when the pane has shrunk
            // away rather than when it starts to.
            let progress = standing.prims.progress();
            let view = standing.view.as_ref().filter(|_| progress > 0.0);
            let drawn = standing.drawn.unwrap_or(target);
            let tether = view.zip(standing.to).map(|(view, to)| {
                let pane = Pane {
                    centre:     drawn.translation + Vec3::new(0.0, view.size.y / 2.0, 0.0),
                    rotation:   drawn.rotation,
                    half_width: view.size.x / 2.0 * pop::scale(progress),
                };
                tether::tether(&pane, to, tuning)
            });
            let color = standing.accent.unwrap_or(palette.tether);
            standing
                .tether
                .update(doc, tether, color, palette.tether_glow)?;
        }
        Ok(())
    }

    fn get_mut(&mut self, id: StandId) -> Option<&mut Standing> {
        self.slots.get_mut(id.0)?.as_mut()
    }
}

/// Eases a drawn pose toward the one the script puts it at, so a placard
/// follows the eye between fixed ticks rather than stepping across the view.
/// The first pose is taken as it is, since arriving is not a move.
fn eased(from: Option<Transform>, to: Transform, speed: f32, delta: f32) -> Transform {
    let Some(from) = from else {
        return to;
    };
    let step = (speed * delta).clamp(0.0, 1.0);
    let translation = from.translation.lerp(to.translation, step);
    let start = yaw(from.rotation);
    let goal = yaw(to.rotation);
    let turn = (goal - start + PI).rem_euclid(TAU) - PI;
    if from.translation.distance(to.translation) > SNAP_DISTANCE || turn.abs() > SNAP_TURN {
        return to;
    }
    let settled =
        translation.distance(to.translation) <= SETTLE_DISTANCE && turn.abs() <= SETTLE_TURN;
    if settled {
        return Transform {
            rotation: Quat::from_rotation_y(goal),
            ..to
        };
    }
    Transform {
        translation,
        rotation: Quat::from_rotation_y(turn.mul_add(step, start).rem_euclid(TAU)),
        scale: Vec3::ONE,
    }
}

/// The turn of a yaw-only quaternion, in radians.
fn yaw(rotation: Quat) -> f32 {
    2.0 * rotation.y.atan2(rotation.w)
}

/// Turns about the vertical axis so a pane at `at` has its front to `eye`; the
/// way the viewer faces when they are directly over it.
#[must_use]
pub fn turned_toward(at: Vec3, eye: &Transform) -> Quat {
    let across = Vec3::new(at.x - eye.translation.x, 0.0, at.z - eye.translation.z);
    let forward = across.normalize_or_zero();
    if forward == Vec3::ZERO {
        return viewer::yaw_only(viewer::facing(eye));
    }
    viewer::yaw_only(forward)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eye_at(translation: Vec3) -> Transform {
        Transform {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Where the pane's back points, which is the way the viewer looks when
    /// it faces them.
    fn looks(rotation: Quat) -> Vec3 {
        rotation * Vec3::new(0.0, 0.0, -1.0)
    }

    fn near(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1.0e-5
    }

    #[test]
    fn it_faces_the_viewer_from_wherever_they_stand() {
        let at = Vec3::new(1.0, 3.0, -2.0);
        for eye in [
            Vec3::new(1.0, 1.6, 2.0),
            Vec3::new(-4.0, 0.0, -2.0),
            Vec3::new(5.0, 9.0, -2.0),
            Vec3::new(1.0, 0.0, -9.0),
        ] {
            let away = Vec3::new(at.x - eye.x, 0.0, at.z - eye.z).normalize();
            let rotation = turned_toward(at, &eye_at(eye));
            assert!(near(looks(rotation), away), "from {eye:?}");
        }
    }

    #[test]
    fn it_stays_upright_whatever_the_height_difference() {
        let rotation = turned_toward(Vec3::new(0.0, 50.0, -3.0), &eye_at(Vec3::ZERO));
        assert!(rotation.x.abs() < 1.0e-6 && rotation.z.abs() < 1.0e-6);
    }

    #[test]
    fn directly_over_the_eye_it_takes_the_way_the_viewer_faces() {
        let eye = Transform {
            translation: Vec3::new(2.0, 1.6, 2.0),
            rotation:    viewer::yaw_only(Vec3::new(1.0, 0.0, 0.0)),
            scale:       Vec3::ONE,
        };
        let rotation = turned_toward(Vec3::new(2.0, 4.0, 2.0), &eye);
        assert!(near(looks(rotation), Vec3::new(1.0, 0.0, 0.0)));
    }

    fn pose(translation: Vec3, turn: f32) -> Transform {
        Transform {
            translation,
            rotation: Quat::from_rotation_y(turn),
            scale: Vec3::ONE,
        }
    }

    #[test]
    fn a_new_placard_is_drawn_where_it_stands() {
        let target = pose(Vec3::new(1.0, 2.0, 3.0), 0.5);
        assert_eq!(eased(None, target, 18.0, 1.0 / 60.0), target);
    }

    #[test]
    fn a_moved_placard_eases_without_overshooting() {
        let target = pose(Vec3::new(1.0, 2.0, 3.0), 0.5);
        let mut drawn = pose(Vec3::new(0.7, 2.0, 3.0), 0.45);
        let mut last = drawn.translation.distance(target.translation);
        for _ in 0..120 {
            let next = eased(Some(drawn), target, 18.0, 1.0 / 60.0);
            let now = next.translation.distance(target.translation);
            assert!(now <= last + 1.0e-6, "eased away from the target");
            last = now;
            drawn = next;
        }
        assert!(
            near(drawn.translation, target.translation),
            "did not settle"
        );
        assert!(
            near(looks(drawn.rotation), looks(target.rotation)),
            "wrong turn"
        );
    }

    #[test]
    fn a_far_hop_snaps_rather_than_flying_across_the_view() {
        let target = pose(Vec3::new(5.0, 2.0, 3.0), 1.0);
        let from = pose(Vec3::ZERO, 0.0);
        assert_eq!(eased(Some(from), target, 18.0, 1.0 / 60.0), target);
    }

    #[test]
    fn a_settled_placard_snaps_exactly_onto_its_target() {
        let target = pose(Vec3::new(1.0, 2.0, 3.0), 0.5);
        let almost = Transform {
            translation: target.translation + Vec3::splat(SETTLE_DISTANCE / 2.0),
            ..target
        };
        assert_eq!(eased(Some(almost), target, 18.0, 1.0 / 60.0), target);
    }
}
