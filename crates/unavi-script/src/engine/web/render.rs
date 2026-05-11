use std::sync::{Arc, atomic::Ordering};

use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use crate::RenderTicking;

use super::{init::InitializedScript, instantiate::ScriptGuest};

pub fn render_tick_scripts(
    to_tick: Query<(&RenderTicking, &ScriptGuest), With<InitializedScript>>,
) {
    for (ticking, guest) in to_tick {
        if ticking.0.swap(true, Ordering::Relaxed) {
            continue;
        }

        let ticking = Arc::clone(&ticking.0);
        let guest = Arc::clone(&guest.0);

        spawn_async_task(async move {
            guest.render().await;
            ticking.store(false, Ordering::Relaxed);
        });
    }
}
