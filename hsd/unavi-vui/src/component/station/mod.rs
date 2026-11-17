//! Stations: panes that open and close together, arranged by a tiling.
//!
//! A station is planted where the viewer stood when it was summoned, and its
//! panes stand on an arc around that point. A pane pulled out by a grip
//! floats, leaving its place empty; let go near the station it goes back to
//! the nearest place, and let go far from it stays where it was left.

use wired_guest::math::{
    Transform,
    Vec3,
};

use self::{
    arc::Arc,
    dock::Screen,
    stage::Stage,
    tiling::{
        Cells,
        Slot,
        Tiling,
    },
};
use crate::{
    component::pane::{
        COLUMN,
        LINE,
    },
    part::viewer,
    tokens::tuning::Tuning,
};

pub mod arc;
pub mod dock;
pub mod stage;
pub mod stations;
pub mod tiling;

#[cfg(test)] mod tests;

/// A pane pulled out of its place, and whether a hand had it last tick.
#[derive(Debug, Clone, Copy)]
struct Afloat<T> {
    id:   T,
    held: bool,
}

/// One leaf as laid out: its panes, and where the one shown stands.
#[derive(Debug, Clone)]
struct Place<T> {
    tabs:  Vec<T>,
    /// The pane in place, or none while every pane of the leaf is afloat.
    shown: Option<T>,
    pose:  Transform,
}

pub struct Station<T> {
    tiling:  Tiling<T>,
    shown:   bool,
    /// Whether its panes lie against the local screen rather than standing on
    /// the arc, which only a screen the user is at can answer.
    docked:  bool,
    /// Where the viewer stood when it was summoned, and which way they faced.
    planted: Option<Transform>,
    afloat:  Vec<Afloat<T>>,
    /// Where each pane in place stood last tick, which a pull is made from.
    poses:   Vec<(T, Transform)>,
}

impl<T: Copy + PartialEq> Station<T> {
    #[must_use]
    pub const fn new(tiling: Tiling<T>) -> Self {
        Self {
            tiling,
            shown: false,
            docked: false,
            planted: None,
            afloat: Vec::new(),
            poses: Vec::new(),
        }
    }

    /// Puts it up where the viewer is standing now, facing as they face.
    pub const fn summon(&mut self) {
        self.shown = true;
        self.planted = None;
    }

    pub const fn dismiss(&mut self) {
        self.shown = false;
    }

    /// Lays its panes against the edges of the local screen from now on, which
    /// a headset has none of: there it stands on the arc as before.
    pub const fn dock(&mut self) {
        self.docked = true;
    }

    /// Stands its panes on the arc again, wherever they are.
    pub const fn stand(&mut self) {
        self.docked = false;
    }

    #[must_use]
    pub const fn is_shown(&self) -> bool {
        self.shown
    }

    /// Every pane the station arranges, whether in place or afloat.
    #[must_use]
    pub fn members(&self) -> Vec<T> {
        self.tiling.items()
    }

    /// Drops a pane that was freed, so its place goes with it. A station left
    /// with no panes is down.
    pub fn forget(&mut self, id: &T) {
        self.tiling.remove(id);
        self.afloat.retain(|afloat| afloat.id != *id);
        self.poses.retain(|(held, _)| held != id);
        if self.tiling.is_empty() {
            self.shown = false;
        }
    }

    /// Takes every pane in place off its mount, as the station goes away.
    /// Panes afloat are left where they are.
    pub fn clear<S: Stage<T>>(&self, stage: &mut S) {
        for id in self.members() {
            if !self.is_afloat(&id) {
                stage.show(id, false);
                stage.unmount(id);
            }
        }
    }

    fn is_afloat(&self, id: &T) -> bool {
        self.afloat.iter().any(|afloat| afloat.id == *id)
    }

    /// Reads what happened to the panes since the last tick and lays them
    /// out again. Call from `fixed_update`, where state belongs.
    pub fn step<S: Stage<T>>(
        &mut self,
        stage: &mut S,
        eye: &Transform,
        tuning: &Tuning,
        screen: Option<Screen>,
    ) {
        if !self.shown {
            for id in self.members() {
                stage.show(id, false);
            }
            return;
        }
        let origin = *self.planted.get_or_insert_with(|| plant(eye));

        self.pull(stage);
        let mut places = self.places(stage, eye, &origin, tuning, screen);
        while let Some(released) = self.released(stage) {
            let Some(at) = stage.position(released) else {
                continue;
            };
            let near = nearest(places.iter().map(|place| place.pose.translation), at);
            let Some(place) = near
                .filter(|(_, away)| *away <= tuning.station_drop)
                .and_then(|(index, _)| places[index].tabs.first().copied())
            else {
                continue;
            };
            self.tiling.join(released, &place);
            self.afloat.retain(|afloat| afloat.id != released);
            places = self.places(stage, eye, &origin, tuning, screen);
        }

        self.poses.clear();
        for place in &places {
            for tab in &place.tabs {
                if place.shown == Some(*tab) {
                    stage.hang(*tab, place.pose);
                    stage.show(*tab, true);
                    self.poses.push((*tab, place.pose));
                } else {
                    stage.show(*tab, self.is_afloat(tab));
                }
            }
        }
    }

    /// Floats each pane a grip pulled, from where it hung.
    fn pull<S: Stage<T>>(&mut self, stage: &mut S) {
        for (id, pose) in &self.poses {
            if self.afloat.iter().any(|afloat| afloat.id == *id) {
                continue;
            }
            if let Some(by) = stage.take_pulled(*id) {
                stage.carry(*id, pose.translation, by);
                self.afloat.push(Afloat {
                    id:   *id,
                    held: true,
                });
            }
        }
    }

    /// A floating pane a hand has let go of since the last call, if any.
    fn released<S: Stage<T>>(&mut self, stage: &S) -> Option<T> {
        for afloat in &mut self.afloat {
            let now = stage.carried(afloat.id);
            let was = std::mem::replace(&mut afloat.held, now);
            if was && !now {
                return Some(afloat.id);
            }
        }
        None
    }

    /// Each leaf, with the pane that is in place in it and where it stands.
    ///
    /// On a screen the station docks to, each leaf stands where its pixels
    /// fall in the eye's frame, so it follows the eye; everywhere else the
    /// arc's poses are turned by where the station was planted.
    fn places<S: Stage<T>>(
        &self,
        stage: &S,
        eye: &Transform,
        origin: &Transform,
        tuning: &Tuning,
        screen: Option<Screen>,
    ) -> Vec<Place<T>> {
        let region = self.tiling.natural(|id| {
            let (w, h) = stage.cells(*id);
            Cells { w, h }
        });
        let leaves = self.tiling.leaves(region);
        let slots = leaves.iter().map(|leaf| leaf.slot).collect::<Vec<Slot>>();
        let poses: Vec<Transform> = match (self.docked, screen) {
            (true, Some(screen)) => {
                let (depth, grain) = dock::grain(
                    COLUMN * tuning.pane_em,
                    LINE * tuning.pane_em,
                    &screen,
                    tuning.pane_reading,
                );
                let view = Cells {
                    w: screen.pixels.w / grain.column.max(1),
                    h: screen.pixels.h / grain.line.max(1),
                };
                leaves
                    .iter()
                    .map(|leaf| {
                        let size = leaf.tabs.first().map_or_else(Cells::default, |id| {
                            let (w, h) = stage.cells(*id);
                            Cells { w, h }
                        });
                        let rect = dock::dock(leaf.slot, size, region, view, grain);
                        dock::place(rect, &screen, depth, eye)
                    })
                    .collect()
            }
            _ => Arc::of(tuning)
                .poses(&slots, region)
                .into_iter()
                .map(|pose| *origin * pose)
                .collect(),
        };
        leaves
            .into_iter()
            .zip(poses)
            .map(|(leaf, pose)| Place {
                tabs:  leaf.tabs.to_vec(),
                shown: leaf.tabs.iter().find(|id| !self.is_afloat(id)).copied(),
                pose,
            })
            .collect()
    }
}

/// Where a station summoned by `eye` is planted: at the eye, turned to where
/// it faces on the ground plane.
fn plant(eye: &Transform) -> Transform {
    Transform {
        translation: eye.translation,
        rotation:    viewer::yaw_only(viewer::facing(eye)),
        scale:       Vec3::ONE,
    }
}

/// The nearest of `points` to `at` and how far it is.
fn nearest(points: impl Iterator<Item = Vec3>, at: Vec3) -> Option<(usize, f32)> {
    points
        .map(|point| (point - at).length())
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
}
