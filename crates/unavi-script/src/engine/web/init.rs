use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use unavi_util::async_task::spawn_async_task;

use super::instantiate::ScriptGuest;

#[derive(Component)]
pub struct InitializedScript;

type DoneCell = Arc<Mutex<Option<()>>>;

#[derive(Component)]
pub struct InitingScript(DoneCell);

pub fn init_scripts(
    to_init: Query<(Entity, &ScriptGuest), (Without<InitingScript>, Without<InitializedScript>)>,
    mut commands: Commands,
) {
    for (entity, guest) in to_init {
        let cell = Arc::new(Mutex::new(None));
        let guest = Arc::clone(&guest.0);
        let done = Arc::clone(&cell);

        spawn_async_task(async move {
            guest.init().await;
            *done.lock().expect("mutex poisoned") = Some(());
        });

        commands.entity(entity).insert(InitingScript(cell));
    }
}

pub fn poll_initing_scripts(initing: Query<(Entity, &InitingScript)>, mut commands: Commands) {
    for (entity, cell) in initing {
        if cell.0.lock().expect("mutex poisoned").take().is_some() {
            commands
                .entity(entity)
                .remove::<InitingScript>()
                .insert(InitializedScript);
        }
    }
}
