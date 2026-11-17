//! The `pane` resource: a framed page of blocks, standing or floating where
//! the script puts it.

use wired_guest::math::{
    Color,
    Transform,
    Vec2,
    Vec3,
};

use crate::{
    api::{
        drive,
        put_up,
    },
    component::pane::{
        Pane,
        Refused,
        panes::PaneId,
    },
    exports::unavi::vui::api::{
        Block as ScriptBlock,
        GuestPane,
        Hand,
        PaneEvent,
    },
    wired::{
        core::error::Error,
        input::types::PointerKind,
    },
};

mod convert;

pub struct PaneRes(pub PaneId);

fn refused(refused: &Refused) -> Error {
    Error::InvalidArgument(refused.to_string())
}

impl GuestPane for PaneRes {
    fn new(columns: u32, lines: Option<u32>) -> Result<Self, Error> {
        let pane = Pane::new(columns, lines).map_err(|why| refused(&why))?;
        Ok(Self(put_up(|vui| vui.pane(pane))?))
    }

    fn set_title(&self, title: String) -> Result<(), Error> {
        let mut result = Ok(());
        drive(|vui| {
            result = vui.pane_title(self.0, &title);
            Ok(())
        })?;
        result.map_err(|why| refused(&why))
    }

    fn set_status(&self, status: Option<String>) -> Result<(), Error> {
        let mut result = Ok(());
        drive(|vui| {
            result = vui.pane_status(self.0, status.as_deref());
            Ok(())
        })?;
        result.map_err(|why| refused(&why))
    }

    fn set_blocks(&self, blocks: Vec<ScriptBlock>) -> Result<(), Error> {
        let blocks = blocks.into_iter().map(convert::block).collect();
        let mut result = Ok(());
        drive(|vui| {
            result = vui.pane_blocks(self.0, blocks);
            Ok(())
        })?;
        result.map_err(|why| refused(&why))
    }

    fn size(&self) -> Vec2 {
        let mut size = Vec2::ZERO;
        if let Err(err) = drive(|vui| {
            size = vui.pane_size(self.0);
            Ok(())
        }) {
            eprintln!("vui: pane size: {err:?}");
        }
        size
    }

    fn events(&self) -> Vec<PaneEvent> {
        let mut events = Vec::new();
        if let Err(err) = drive(|vui| {
            events = vui.pane_drain(self.0);
            Ok(())
        }) {
            eprintln!("vui: pane events: {err:?}");
        }
        events.into_iter().map(convert::event).collect()
    }

    fn set_shown(&self, value: bool) {
        if let Err(err) = drive(|vui| {
            vui.pane_shown(self.0, value);
            Ok(())
        }) {
            eprintln!("vui: pane shown: {err:?}");
        }
    }

    fn set_standing(&self, at: Vec3) {
        if let Err(err) = drive(|vui| {
            vui.pane_standing(self.0, at);
            Ok(())
        }) {
            eprintln!("vui: pane standing: {err:?}");
        }
    }

    fn set_tether(&self, to: Option<Vec3>) {
        if let Err(err) = drive(|vui| {
            vui.pane_tether(self.0, to);
            Ok(())
        }) {
            eprintln!("vui: pane tether: {err:?}");
        }
    }

    fn set_hue(&self, hue: Option<Color>) {
        if let Err(err) = drive(|vui| {
            vui.pane_hue(self.0, hue);
            Ok(())
        }) {
            eprintln!("vui: pane hue: {err:?}");
        }
    }

    fn set_floating(&self, at: Vec3) {
        if let Err(err) = drive(|vui| {
            vui.pane_floating(self.0, at);
            Ok(())
        }) {
            eprintln!("vui: pane floating: {err:?}");
        }
    }

    fn set_hung(&self, to: Option<(u64, u64)>, pose: Transform) {
        if let Err(err) = drive(|vui| {
            vui.pane_hung(self.0, to, pose);
            Ok(())
        }) {
            eprintln!("vui: pane hung: {err:?}");
        }
    }

    fn set_held(&self, hand: Hand) {
        let hand = match hand {
            Hand::Left => PointerKind::LeftHand,
            Hand::Right => PointerKind::RightHand,
        };
        if let Err(err) = drive(|vui| {
            vui.pane_held(self.0, hand);
            Ok(())
        }) {
            eprintln!("vui: pane held: {err:?}");
        }
    }

    fn position(&self) -> Vec3 {
        let mut at = Vec3::ZERO;
        if let Err(err) = drive(|vui| {
            at = vui.pane_position(self.0).unwrap_or_default();
            Ok(())
        }) {
            eprintln!("vui: pane position: {err:?}");
        }
        at
    }
}

/// Frees the pane's prims once nothing holds this handle.
impl Drop for PaneRes {
    fn drop(&mut self) {
        if let Err(err) = drive(|vui| vui.pane_remove(self.0)) {
            eprintln!("vui: could not free a dropped pane: {err:?}");
        }
    }
}
