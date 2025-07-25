use bevy::prelude::*;
use tokio::sync::mpsc::error::TryRecvError;

use crate::load::ScriptCommands;

pub enum WasmCommand {
    RegisterComponent { id: u64 },
    RegisterSystem { id: u64 },
}

pub fn process_commands(mut scripts: Query<&mut ScriptCommands>) {
    for mut recv in scripts.iter_mut() {
        match recv.0.try_recv() {
            Ok(WasmCommand::RegisterComponent { id }) => {
                info!("register c {id}");
            }
            Ok(WasmCommand::RegisterSystem { id }) => {
                info!("register s {id}");
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                warn!("ScriptCommands disconnected");
            }
        }
    }
}
