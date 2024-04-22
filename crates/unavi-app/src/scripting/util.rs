use std::sync::Arc;

use bevy::tasks::futures_lite::future;
use tokio::sync::{Mutex, MutexGuard};

pub fn blocking_lock<T>(item: &Arc<Mutex<T>>) -> MutexGuard<T> {
    future::block_on(async move { item.lock().await })
}
