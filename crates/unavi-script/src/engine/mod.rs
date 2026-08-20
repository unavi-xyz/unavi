use bevy::prelude::*;

use crate::Script;

#[cfg(not(target_family = "wasm"))] mod native;
#[cfg(target_family = "wasm")] mod web;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        cfg_select! {
            target_family = "wasm" => {
                app.add_plugins(web::WebEnginePlugin);
            }
            _ => {
                app.add_plugins(native::NativeEnginePlugin);
            }
        }

        let default_engine = app.world_mut().spawn(Engine).id();

        app.insert_resource(DefaultEngine(default_engine))
            .add_observer(add_to_default_engine);
    }
}

#[derive(Component)]
pub struct InitializedScript;

#[derive(Component)]
#[require(Scripts)]
pub struct Engine;

#[derive(Component, Default)]
#[relationship_target(relationship = ScriptEngine)]
pub struct Scripts(Vec<Entity>);

#[derive(Component)]
#[relationship(relationship_target = Scripts)]
pub struct ScriptEngine(pub Entity);

#[derive(Resource)]
pub struct DefaultEngine(Entity);

fn add_to_default_engine(
    trigger: On<Add, Script>,
    default_engine: Res<DefaultEngine>,
    engines: Query<&ScriptEngine>,
    mut commands: Commands,
) {
    if engines.contains(trigger.entity) {
        return;
    }
    commands
        .entity(trigger.entity)
        .insert(ScriptEngine(default_engine.0));
}
