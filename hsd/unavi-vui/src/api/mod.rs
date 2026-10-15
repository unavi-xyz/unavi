//! The `unavi:vui` interface: the whole of VUI as a script sees it.

use std::cell::RefCell;

use crate::{
    World,
    exports::unavi::vui::api::{
        Event,
        Guest,
    },
    render::{
        SurfaceId,
        Vui,
    },
    wired::core::error::Error,
};

mod convert;
mod mote;
mod surface;

thread_local! {
    /// Every surface this script has put up, built on the first one.
    static VUI: RefCell<Option<Vui>> = const { RefCell::new(None) };
}

impl Guest for World {
    type Grid = surface::Grid;
    type Mote = mote::MoteRes;
    type Orbit = surface::Orbit;

    fn fixed_update() -> Result<(), Error> {
        drive(Vui::fixed_update)
    }

    fn update(dt: f32) -> Result<(), Error> {
        drive(|vui| vui.update(dt))
    }
}

fn put_up(shape: impl FnOnce(&mut Vui) -> anyhow::Result<SurfaceId>) -> Result<SurfaceId, Error> {
    VUI.with_borrow_mut(|slot| {
        let vui = match slot {
            Some(vui) => vui,
            None => slot.insert(Vui::new().map_err(|err| failed(&err))?),
        };
        shape(vui).map_err(|err| failed(&err))
    })
}

fn drain(surface: SurfaceId) -> Vec<Event> {
    VUI.with_borrow_mut(|vui| {
        vui.as_mut()
            .map(|vui| vui.drain(surface))
            .unwrap_or_default()
            .into_iter()
            .map(convert::event)
            .collect()
    })
}

fn summon(surface: SurfaceId) -> Result<(), Error> {
    drive(|vui| vui.summon(surface))
}

fn dismiss(surface: SurfaceId) -> Result<(), Error> {
    drive(|vui| vui.dismiss(surface))
}

/// Frees a surface's prims. Called when the `orbit`/`grid` handle drawing it
/// is dropped, so nothing the script let go of keeps costing frames.
fn remove(surface: SurfaceId) {
    if let Err(err) = drive(|vui| vui.remove(surface)) {
        eprintln!("vui: could not free a dropped surface: {err:?}");
    }
}

fn shown(surface: SurfaceId) -> bool {
    VUI.with_borrow(|vui| vui.as_ref().is_some_and(|vui| vui.is_shown(surface)))
}

/// Driving before anything is put up is not an error: there is nothing drawn
/// to step.
fn drive(step: impl FnOnce(&mut Vui) -> anyhow::Result<()>) -> Result<(), Error> {
    VUI.with_borrow_mut(|slot| {
        slot.as_mut()
            .map_or(Ok(()), |vui| step(vui).map_err(|err| failed(&err)))
    })
}

fn failed(err: &anyhow::Error) -> Error {
    Error::Internal(format!("{err:?}"))
}
