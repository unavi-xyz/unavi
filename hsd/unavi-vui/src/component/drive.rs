//! Drives the cast site a tapped mote opens, from the grasp that holds it.

use wired_guest::math::Vec3;

use crate::{
    component::{
        event::{
            Casting,
            Event,
        },
        site::Site,
        surface::Surface,
        tree::Mote,
    },
    machinery::cast::{
        Cast,
        State,
    },
    wired::scene::document::Document,
};

/// Opens a cast site on the mote that was tapped, whichever panel shows it.
pub fn open_cast(casting: &mut Option<Casting>, slot: usize, mote: Mote, surface: &Surface) {
    *casting = Some(Casting {
        slot,
        mote,
        cast: Cast::standard(surface.tuning()),
    });
}

/// Fills the open cast site while the grasp stays down on the mote that opened
/// it, and aborts the moment it lets go.
///
/// The hold is the confirmation, so it is the grasp that fills the ring rather
/// than attention. Filling on attention made a cast fire from a single click:
/// the pointer is still on the mote it just pressed, so the ring filled with
/// no further input and the hold was decorative.
pub fn drive_cast(
    doc: &Document,
    casting: &mut Option<Casting>,
    surface: &Surface,
    site: &Site,
    delta: f32,
    events: &mut Vec<Event>,
) -> anyhow::Result<()> {
    // Before the guard: an abandoned ring is unwinding precisely when there is
    // no cast left to drive.
    site.step(doc, delta)?;

    let Some(active) = casting else {
        return Ok(());
    };

    let held = surface.seized_slot() == Some(active.slot);
    let state = active.cast.update(held, delta);
    let (at, radius) = surface.views().get(active.slot).map_or_else(
        || (Vec3::ZERO, surface.tuning().mote_radius),
        |view| (view.position, view.radius),
    );
    site.apply(doc, at, radius, state, active.mote.tint())?;

    if !state.is_settled() {
        return Ok(());
    }
    let mote = active.mote.clone();
    *casting = None;
    events.push(match state {
        State::Committed => Event::Cast(mote),
        State::Filling(_) | State::Aborted => Event::Aborted(mote),
    });
    Ok(())
}
