use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use super::{init::InitializedScript, instantiate::ScriptGuest};
use crate::Ticking;

const TICKRATE: Duration = Duration::from_millis(50);

#[derive(Component, Default)]
pub struct LastTick(Duration);

pub fn tick_scripts(
    time: Res<Time>,
    to_tick: Query<(&Ticking, &ScriptGuest, &mut LastTick), With<InitializedScript>>,
) {
    let now = time.elapsed();

    for (ticking, guest, mut last) in to_tick {
        let delta = now.checked_sub(last.0).unwrap_or_default();
        if delta < TICKRATE {
            continue;
        }
        if ticking.0.swap(true, Ordering::SeqCst) {
            continue;
        }

        let margin = delta
            .checked_sub(TICKRATE)
            .expect("always greater")
            .min(TICKRATE);
        last.0 = now.checked_sub(margin).unwrap_or_default();

        let ticking = Arc::clone(&ticking.0);
        let guest = Arc::clone(&guest.0);

        spawn_async_task(async move {
            guest.tick().await;
            ticking.store(false, Ordering::SeqCst);
        });
    }
}
