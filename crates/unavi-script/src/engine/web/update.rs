use std::sync::{
    Arc,
    atomic::Ordering,
};

use bevy::prelude::*;
use bevy_async::task;

use super::instantiate::ScriptGuest;
use crate::{
    FixedUpdating,
    engine::InitializedScript,
};

pub fn update_scripts(to_update: Query<(&FixedUpdating, &ScriptGuest), With<InitializedScript>>) {
    // Ensures only one update call at a time; native relies on a lock instead.
    for (updating, guest) in to_update {
        if updating.0.swap(true, Ordering::SeqCst) {
            continue;
        }

        let updating = Arc::clone(&updating.0);
        let guest = Arc::clone(&guest.0);

        task::spawn(async move {
            guest.update().await;
            updating.store(false, Ordering::SeqCst);
        });
    }
}
