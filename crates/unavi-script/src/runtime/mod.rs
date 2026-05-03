use bevy::prelude::*;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

pub mod shared;

#[cfg(not(target_family = "wasm"))]
pub mod native;
#[cfg(target_family = "wasm")]
pub mod web;

pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(
            shared::wired::input::bridge::bridge_squeeze_down
                .pipe(shared::wired::input::bridge::send_to_listeners),
        )
        .add_observer(
            shared::wired::input::bridge::bridge_squeeze_up
                .pipe(shared::wired::input::bridge::send_to_listeners),
        );
    }
}

#[cfg_attr(target_family = "wasm", wasm_bindgen(getter_with_clone))]
#[cfg_attr(target_family = "wasm", derive(Clone))]
pub struct Runtime {
    pub(crate) backend: shared::RuntimeBackend,
    #[cfg(not(target_family = "wasm"))]
    pub native: native::NativeRuntime,
}
