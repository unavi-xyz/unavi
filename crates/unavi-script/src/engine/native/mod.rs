//! The wasmtime engine: one compiled component per distinct script, a store
//! per instance, and the lifecycle driven from the frame.

use std::sync::Arc;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use parking_lot::Mutex;
use wasmtime::{
    Config,
    component::Linker,
};

use crate::{
    ScriptSystems,
    bindings::native::{
        HostCtx,
        ShellPre,
        linker,
    },
    engine::TickKind,
};

mod drive;
mod instantiate;
mod log;
mod tick;

pub struct NativeEnginePlugin;

impl Plugin for NativeEnginePlugin {
    fn build(&self, app: &mut App) {
        match WasmtimeEngine::new() {
            Ok(engine) => {
                app.insert_resource(engine);
            }
            Err(err) => {
                error!(
                    ?err,
                    "Failed to create the wasmtime engine; no script will run"
                );
                return;
            }
        }

        app.add_systems(PreUpdate, increment_epoch)
            .add_systems(
                Update,
                tick::drive::<{ tick::UPDATE }>.in_set(ScriptSystems::Tick),
            )
            .add_systems(
                FixedUpdate,
                (
                    instantiate::instantiate_scripts,
                    instantiate::finish_instantiating,
                    tick::drive::<{ tick::FIXED }>,
                )
                    .chain(),
            );
    }
}

/// The engine, a linker holding every host interface, and each distinct
/// script compiled once, by the hash of its bytes.
#[derive(Resource, Clone)]
pub struct WasmtimeEngine {
    engine:   wasmtime::Engine,
    linker:   Arc<Linker<HostCtx>>,
    compiled: Arc<Mutex<HashMap<blake3::Hash, ShellPre<HostCtx>>>>,
}

impl WasmtimeEngine {
    fn new() -> wasmtime::Result<Self> {
        let engine = wasmtime::Engine::new(Config::default().epoch_interruption(true))?;
        let linker = Arc::new(linker(&engine)?);
        Ok(Self {
            engine,
            linker,
            compiled: Arc::default(),
        })
    }

    /// The compiled, linked form of `bytes`, compiling it on first sight.
    /// Compiling untrusted bytes is the expensive step, so it happens once
    /// per distinct script however many instances it has.
    fn prepare(&self, hash: blake3::Hash, bytes: &[u8]) -> wasmtime::Result<ShellPre<HostCtx>> {
        if let Some(pre) = self.compiled.lock().get(&hash) {
            return Ok(pre.clone());
        }
        let component = wasmtime::component::Component::from_binary(&self.engine, bytes)?;
        let pre = ShellPre::new(self.linker.instantiate_pre(&component)?)?;
        self.compiled.lock().insert(hash, pre.clone());
        Ok(pre)
    }
}

/// Advances the epoch once a frame, which is what a guest's run time is
/// measured in.
fn increment_epoch(engine: Res<WasmtimeEngine>) {
    engine.engine.increment_epoch();
}

impl TickKind {
    const fn export(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Update => "update",
            Self::FixedUpdate => "fixed-update",
        }
    }
}
