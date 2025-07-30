use bevy::prelude::*;

use crate::WasmEngine;

pub mod init;

pub fn increment_epochs(engines: Query<&WasmEngine>) {
    for engine in engines {
        engine.0.increment_epoch();
    }
}
