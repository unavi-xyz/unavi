//! The `station_*` calls: panes arranged together.

use super::Vui;
use crate::component::{
    pane::panes::PaneId,
    station::{
        stations::StationId,
        tiling::Tiling,
    },
};

impl Vui {
    pub fn station(&mut self, tiling: Tiling<PaneId>) -> StationId {
        self.stations.add(tiling)
    }

    pub fn station_summon(&mut self, id: StationId) {
        self.stations.summon(id);
    }

    pub fn station_dismiss(&mut self, id: StationId) {
        self.stations.dismiss(id);
    }

    #[must_use]
    pub fn station_shown(&self, id: StationId) -> bool {
        self.stations.is_shown(id)
    }

    pub fn station_remove(&mut self, id: StationId) {
        self.stations.remove(&mut self.panes, id);
    }
}
