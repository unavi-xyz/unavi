use bevy::prelude::*;

#[cfg(not(target_family = "wasm"))]
pub mod agent;
#[cfg(not(target_family = "wasm"))]
mod api;
mod asset;
pub mod core_ops;
pub mod event_registry;
pub mod firewall;
pub mod input_registry;
pub mod load;
pub mod permissions;
#[cfg(not(target_family = "wasm"))]
mod runtime;
mod util;
#[cfg(target_family = "wasm")]
mod web;

#[cfg(not(target_family = "wasm"))]
pub use api::wired::scene::GlobalRegistryMapRes;
pub use event_registry::EventRegistry;
pub use input_registry::{InputAction, InputDevice, InputRegistry, QueuedEvent};

pub use load::local::{LoadLocalScript, ScriptSource};
pub use permissions::ScriptPermissions;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>()
            .init_resource::<EventRegistry>()
            .init_resource::<InputRegistry>()
            .add_observer(load::local::on_load_local_script);

        #[cfg(not(target_family = "wasm"))]
        app.add_plugins(native::NativeScriptPlugin);

        #[cfg(target_family = "wasm")]
        app.add_plugins(web::WebScriptPlugin);
    }
}

#[cfg(not(target_family = "wasm"))]
mod native;

#[cfg(not(target_family = "wasm"))]
#[derive(Component)]
#[require(Scripts)]
pub struct WasmEngine(pub wasmtime::Engine);

#[derive(Component, Default)]
#[relationship_target(relationship = ScriptEngine)]
pub struct Scripts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Scripts)]
pub struct ScriptEngine(pub Entity);

#[derive(Component)]
pub struct WasmBinary(pub Handle<asset::Wasm>);
