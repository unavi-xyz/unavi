//! The `station` resource: panes that open and close together, arranged by a
//! tiling.

use super::{
    VUI,
    drive,
    pane::PaneRes,
    put_up,
};
use crate::{
    component::station::stations::StationId,
    exports::unavi::vui::api::{
        GuestStation,
        PaneBorrow,
        Tile,
    },
    wired::core::error::Error,
};

mod convert;

pub struct StationRes(pub StationId);

impl GuestStation for StationRes {
    fn new(panes: Vec<PaneBorrow<'_>>, tiles: Vec<Tile>) -> Result<Self, Error> {
        let panes = panes
            .iter()
            .map(|pane| pane.get::<PaneRes>().0)
            .collect::<Vec<_>>();
        let tiling = convert::tiling(&panes, &tiles)?;
        Ok(Self(put_up(|vui| Ok(vui.station(tiling)))?))
    }

    fn summon(&self) -> Result<(), Error> {
        drive(|vui| {
            vui.station_summon(self.0);
            Ok(())
        })
    }

    fn dismiss(&self) -> Result<(), Error> {
        drive(|vui| {
            vui.station_dismiss(self.0);
            Ok(())
        })
    }

    fn dock(&self) -> Result<(), Error> {
        drive(|vui| {
            vui.station_dock(self.0);
            Ok(())
        })
    }

    fn stand(&self) -> Result<(), Error> {
        drive(|vui| {
            vui.station_stand(self.0);
            Ok(())
        })
    }

    fn shown(&self) -> bool {
        VUI.with_borrow(|vui| {
            vui.as_ref()
                .is_some_and(|vui| vui.station_shown(self.0))
        })
    }
}

/// Frees the station once nothing holds this handle. The panes stay: a script
/// may still hold them.
impl Drop for StationRes {
    fn drop(&mut self) {
        if let Err(err) = drive(|vui| {
            vui.station_remove(self.0);
            Ok(())
        }) {
            eprintln!("vui: could not free a dropped station: {err:?}");
        }
    }
}
