//! Every station a script holds, as a free list like the panes.

use wired_guest::math::Transform;

use super::{
    Station,
    dock::Screen,
    tiling::Tiling,
};
use crate::{
    component::pane::panes::{
        PaneId,
        Panes,
    },
    tokens::tuning::Tuning,
};

/// A station [`Stations`] is holding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StationId(usize);

#[derive(Default)]
pub struct Stations {
    slots: Vec<Option<Station<PaneId>>>,
    free:  Vec<usize>,
}

impl Stations {
    pub fn add(&mut self, tiling: Tiling<PaneId>) -> StationId {
        let station = Station::new(tiling);
        if let Some(index) = self.free.pop() {
            self.slots[index] = Some(station);
            return StationId(index);
        }
        self.slots.push(Some(station));
        StationId(self.slots.len() - 1)
    }

    /// Frees a station and takes the panes in its places down.
    pub fn remove(&mut self, panes: &mut Panes, id: StationId) {
        let Some(station) = self.slots.get_mut(id.0).and_then(Option::take) else {
            return;
        };
        station.clear(panes);
        self.free.push(id.0);
    }

    pub fn summon(&mut self, id: StationId) {
        if let Some(station) = self.get_mut(id) {
            station.summon();
        }
    }

    pub fn dismiss(&mut self, id: StationId) {
        if let Some(station) = self.get_mut(id) {
            station.dismiss();
        }
    }

    #[must_use]
    pub fn is_shown(&self, id: StationId) -> bool {
        self.slots
            .get(id.0)
            .and_then(Option::as_ref)
            .is_some_and(Station::is_shown)
    }

    /// Takes a freed pane out of every station that arranged it, since its
    /// index may be handed to another pane.
    pub fn forget(&mut self, pane: PaneId) {
        for station in self.slots.iter_mut().flatten() {
            station.forget(&pane);
        }
    }

    /// Lays it against the local screen, which a headset has none of.
    pub fn dock(&mut self, id: StationId) {
        if let Some(station) = self.get_mut(id) {
            station.dock();
        }
    }

    /// Stands it on the arc again.
    pub fn stand(&mut self, id: StationId) {
        if let Some(station) = self.get_mut(id) {
            station.stand();
        }
    }

    /// Reads what happened to each station's panes and lays them out again.
    pub fn step(
        &mut self,
        panes: &mut Panes,
        eye: &Transform,
        tuning: &Tuning,
        screen: Option<Screen>,
    ) {
        for station in self.slots.iter_mut().flatten() {
            station.step(panes, eye, tuning, screen);
        }
    }

    fn get_mut(&mut self, id: StationId) -> Option<&mut Station<PaneId>> {
        self.slots.get_mut(id.0)?.as_mut()
    }
}
