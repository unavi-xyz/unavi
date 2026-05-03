use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

use crate::{load::asset::Wasm, permissions::ApiPermissions, registry::DocTransformRegistry};

mod engine;
pub mod firewall;
pub mod load;
pub mod permissions;
pub mod registry;
mod runtime;
mod util;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        let transform_reg = DocTransformRegistry(Arc::new(Mutex::new(HashMap::new())));
        app.insert_resource(transform_reg)
            .add_observer(registry::on_hsd_record_added)
            .add_systems(PostUpdate, registry::sync_outbound_transforms)
            .add_plugins((
                engine::EnginePlugin,
                load::LoadPlugin,
                runtime::RuntimePlugin,
            ));
    }
}

#[derive(Component)]
#[require(ApiPermissions)]
pub struct Script(pub Handle<Wasm>);
