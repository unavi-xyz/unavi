use std::{
    sync::{
        Arc,
        atomic::Ordering,
    },
    time::Duration,
};

use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use super::instantiate::ScriptGuest;
use crate::{
    FixedUpdating,
    engine::InitializedScript,
};

const FIXED_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Component, Default)]
pub struct LastFixedUpdate(Duration);

pub fn fixed_update_scripts(
    time: Res<Time>,
    to_update: Query<(&FixedUpdating, &ScriptGuest, &mut LastFixedUpdate), With<InitializedScript>>,
) {
    let now = time.elapsed();

    for (updating, guest, mut last) in to_update {
        let delta = now.checked_sub(last.0).unwrap_or_default();
        if delta < FIXED_UPDATE_INTERVAL {
            continue;
        }
        if updating.0.swap(true, Ordering::SeqCst) {
            continue;
        }

        let margin = delta
            .checked_sub(FIXED_UPDATE_INTERVAL)
            .expect("always greater")
            .min(FIXED_UPDATE_INTERVAL);
        last.0 = now.checked_sub(margin).unwrap_or_default();

        let updating = Arc::clone(&updating.0);
        let guest = Arc::clone(&guest.0);

        spawn_async_task(async move {
            guest.fixed_update().await;
            updating.store(false, Ordering::SeqCst);
        });
    }
}
