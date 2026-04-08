use bevy::prelude::*;

use crate::{event_registry::EventRegistry, input_registry::InputRegistry};

mod asset;
pub mod core_ops;
pub mod event_registry;
pub mod firewall;
pub mod input_registry;
pub mod load;
pub mod permissions;
mod util;

#[cfg(not(target_family = "wasm"))]
pub mod native;

#[cfg(target_family = "wasm")]
mod web;

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

#[derive(Component, Default)]
#[relationship_target(relationship = ScriptEngine)]
pub struct Scripts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Scripts)]
pub struct ScriptEngine(pub Entity);

#[derive(Component)]
pub struct WasmBinary(pub Handle<asset::Wasm>);
