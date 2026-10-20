//! The `placard` resource: text on a pane standing where the script puts it.

use wired_guest::math::{
    Color,
    Vec2,
    Vec3,
};

use crate::{
    api::{
        drive,
        put_up,
    },
    exports::unavi::vui::api::{
        GuestPlacard,
        PlacardLine,
        Tone as ScriptTone,
    },
    placard::{
        Placard,
        Tone,
    },
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

    fn set_text(
        &self,
        title: String,
        kicker: Option<String>,
        lines: Vec<PlacardLine>,
    ) -> Result<(), Error> {
        let lines = lines
            .into_iter()
            .map(|line| (line.text, tone(line.tone)))
            .collect::<Vec<_>>();
        let text = Placard::from_script(&title, kicker.as_deref(), &lines)
            .map_err(|refused| Error::InvalidArgument(refused.to_string()))?;
        drive(|vui| {
            vui.stand_text(self.0, &text);
            Ok(())
        })
    }

    fn size(&self) -> Vec2 {
        let mut size = Vec2::ZERO;
        let _ = drive(|vui| {
            size = vui.stand_size(self.0);
            Ok(())
        });
        size
    }

    fn set_position(&self, at: Vec3) {
        let _ = drive(|vui| {
            vui.stand_at(self.0, at);
            Ok(())
        });
    }

    fn set_accent(&self, accent: Option<Color>) {
        let _ = drive(|vui| {
            vui.stand_accent(self.0, accent);
            Ok(())
        });
    }

    fn set_tether(&self, to: Option<Vec3>) {
        let _ = drive(|vui| {
            vui.stand_tether(self.0, to);
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

const fn tone(tone: ScriptTone) -> Tone {
    match tone {
        ScriptTone::Body => Tone::Body,
        ScriptTone::Muted => Tone::Muted,
        ScriptTone::Warning => Tone::Warning,
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
