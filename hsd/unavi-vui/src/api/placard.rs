//! The `placard` resource: text on a pane standing where the script puts it.

use wired_guest::math::Vec3;

use crate::{
    api::{
        drive,
        put_up,
    },
    exports::unavi::vui::api::GuestPlacard,
    placard::Placard,
    render::{
        Vui,
        stand::StandId,
    },
    wired::core::error::Error,
};

pub struct PlacardRes(StandId);

impl GuestPlacard for PlacardRes {
    fn new() -> Result<Self, Error> {
        Ok(Self(put_up(Vui::stand)?))
    }

    fn set_text(&self, title: String, lines: Vec<String>) -> Result<(), Error> {
        let text = Placard::from_script(&title, &lines)
            .map_err(|refused| Error::InvalidArgument(refused.to_string()))?;
        drive(|vui| {
            vui.stand_text(self.0, &text);
            Ok(())
        })
    }

    fn set_position(&self, at: Vec3) {
        let _ = drive(|vui| {
            vui.stand_at(self.0, at);
            Ok(())
        });
    }

    fn set_shown(&self, value: bool) {
        let _ = drive(|vui| {
            vui.stand_shown(self.0, value);
            Ok(())
        });
    }
}

/// Frees the placard's prims once nothing holds this handle.
impl Drop for PlacardRes {
    fn drop(&mut self) {
        if let Err(err) = drive(|vui| vui.stand_remove(self.0)) {
            eprintln!("vui: could not free a dropped placard: {err:?}");
        }
    }
}
