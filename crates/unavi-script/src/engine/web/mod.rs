//! Runs scripts by transpiling them through `jco` and driving the result
//! from JS.
//!
//! **No CPU or memory guard.** Unlike `engine::native`, which traps a guest
//! past an epoch ceiling and charges its linear memory against a quota,
//! nothing here can interrupt a synchronous guest loop or bound how much
//! memory one allocates: there is no `wasmtime`-style limiter for a
//! `jco`-transpiled module running on the page's own thread. A script that
//! spins synchronously freezes the tab exactly as any other runaway script
//! would.

use bevy::prelude::*;

mod instantiate;
mod tick;

pub struct WebEnginePlugin;

impl Plugin for WebEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            tick::drive::<{ tick::UPDATE }>.in_set(crate::ScriptSystems::Tick),
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
