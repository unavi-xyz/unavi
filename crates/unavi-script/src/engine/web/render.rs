use std::sync::{
    Arc,
    atomic::Ordering,
};

use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use super::{
    init::InitializedScript,
    instantiate::ScriptGuest,
};
use crate::Ticking;

pub fn render_tick_scripts(to_tick: Query<(&Ticking, &ScriptGuest), With<InitializedScript>>) {
    // Use [`Ticking`] not [`RenderTicking`], to enforce we only every call one at a
    // time. We use RenderTicking on native because a lock ensures sequential
    // execution regardless.
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
