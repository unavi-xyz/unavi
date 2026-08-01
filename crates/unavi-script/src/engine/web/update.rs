use std::sync::{
    Arc,
    atomic::Ordering,
};

use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use super::instantiate::ScriptGuest;
use crate::{
    FixedUpdating,
    engine::InitializedScript,
};

pub fn update_scripts(to_update: Query<(&FixedUpdating, &ScriptGuest), With<InitializedScript>>) {
    // Use [`FixedUpdating`] to ensure only one call at a time; native relies on
    // a lock for sequential execution instead.
    for (updating, guest) in to_update {
        if updating.0.swap(true, Ordering::SeqCst) {
            continue;
        }

        let updating = Arc::clone(&updating.0);
        let guest = Arc::clone(&guest.0);

        spawn_async_task(async move {
            guest.update().await;
            updating.store(false, Ordering::SeqCst);
        });
    }
}
