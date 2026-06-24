use std::sync::{
    Arc,
    atomic::Ordering,
};

use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use super::instantiate::ScriptGuest;
use crate::{
    Ticking,
    engine::InitializedScript,
};

pub fn render_tick_scripts(to_tick: Query<(&Ticking, &ScriptGuest), With<InitializedScript>>) {
    // Use [`Ticking`] not [`RenderTicking`] to ensure only one call at a time;
    // native relies on a lock for sequential execution instead.
    for (ticking, guest) in to_tick {
        if ticking.0.swap(true, Ordering::SeqCst) {
            continue;
        }

        let ticking = Arc::clone(&ticking.0);
        let guest = Arc::clone(&guest.0);

        spawn_async_task(async move {
            guest.render().await;
            ticking.store(false, Ordering::SeqCst);
        });
    }
}
