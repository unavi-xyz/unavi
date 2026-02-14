use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use wasmtime::{Store, component::ResourceAny};

use crate::{
    WasmEngine,
    load::{
        log::{ScriptStderr, ScriptStdout},
        state::StoreState,
    },
};

pub mod init;
mod log;
pub mod tick;

pub struct RuntimeCtx {
    pub store: Store<StoreState>,
    stdout: ScriptStdout,
    stderr: ScriptStderr,
    pub script: Option<ResourceAny>,
    last_tick: Duration,
}

impl RuntimeCtx {
    pub async fn flush_logs(&mut self) {
        let mut buf = [0; 1024];
        if let Some(s) = log::try_read_text_stream(&mut buf, &mut self.stdout.0).await {
            for line in s.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    info!("{line}");
                }
            }
        }
        if let Some(s) = log::try_read_text_stream(&mut buf, &mut self.stderr.0).await {
            for line in s.lines() {
                let line = line.trim();
                if !line.is_empty() {
                    error!("{line}");
                }
            }
        }
    }
}

#[derive(Component)]
#[require(tick::TickingTask)]
pub struct ScriptRuntime {
    pub ctx: Arc<tokio::sync::Mutex<RuntimeCtx>>,
}

impl ScriptRuntime {
    pub fn new(store: Store<StoreState>, stdout: ScriptStdout, stderr: ScriptStderr) -> Self {
        Self {
            ctx: Arc::new(tokio::sync::Mutex::new(RuntimeCtx {
                store,
                stdout,
                stderr,
                script: None,
                last_tick: Duration::from_secs(0),
            })),
        }
    }
}

pub fn increment_epochs(engines: Query<&WasmEngine>) {
    for engine in engines {
        engine.0.increment_epoch();
    }
}
