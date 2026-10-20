//! Placards a script stands where it likes, rather than on a mote.

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

    /// Turns each placard toward `eye`, steps its pop, and lays its tether
    /// from wherever the pane has turned to.
    pub fn update(
        &self,
        doc: &Document,
        eye: &Transform,
        palette: &Palette,
        tuning: &Tuning,
        delta: f32,
    ) -> anyhow::Result<()> {
        for standing in self.slots.iter().flatten() {
            let wanted = standing.view.as_ref().filter(|_| standing.shown);
            let rotation = turned_toward(standing.at, eye);
            if wanted.is_some()
                && let Some(pose) = standing.pose.diff(Transform {
                    translation: standing.at,
                    rotation,
                    scale: Vec3::ONE,
                })
            {
                doc.local()
                    .set(standing.root, Property::Transform(pose))
                    .flush()?;
            }
            standing.prims.update(doc, wanted, palette, delta)?;

            // The tether pops with the pane: it reaches the pane's edge as the
            // pane grows from its centre, and goes when the pane has shrunk
            // away rather than when it starts to.
            let progress = standing.prims.progress();
            let drawn = standing.view.as_ref().filter(|_| progress > 0.0);
            let tether = drawn.zip(standing.to).map(|(view, to)| {
                let pane = Pane {
                    centre: standing.at + Vec3::new(0.0, view.size.y / 2.0, 0.0),
                    rotation,
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
}
