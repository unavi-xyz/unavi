//! Picking up and carrying a dynamic body with a pointer.
//!
//! [`GrabPlugin`] turns a grip into velocity on whatever the grip found —
//! input to physics, nothing else. It reads [`unavi_input::action::Action`]
//! and avian's bodies only; claiming anything beyond physics (a document's
//! hold, say) is for an observer on [`events::Grabbed`]/[`events::Released`] in
//! whatever crate owns that claim.

pub mod events;
mod held;
mod pending;

use bevy::prelude::*;

pub struct GrabPlugin;

impl Plugin for GrabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<pending::PendingGrabs>()
            .add_observer(held::on_held_removed)
            .add_observer(pending::on_rigid_body_removed)
            .add_systems(
                Update,
                (
                    pending::on_press,
                    held::on_release,
                    pending::cancel_pending_on_release,
                    pending::note_promoted_bodies,
                    pending::start_pending_grabs,
                    held::reach_grabbed_objects,
                    held::move_grabbed_objects,
                )
                    .chain(),
            );
    }
}
