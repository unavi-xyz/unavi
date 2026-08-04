use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdChild,
    HsdDocId,
    Prim,
};
use tokio::sync::Mutex;
use unavi_quota::Quota;
use unavi_util::async_task::spawn_async_task;

use crate::{
    Script,
    engine::web::fixed_update::LastFixedUpdate,
    load::asset::Wasm,
    permissions::ApiPermissions,
    quota::QuotaExempt,
    runtime::{
        Runtime,
        shared::Api,
        web::{
            ScriptCell,
            ScriptInstance,
        },
    },
};

#[derive(Component)]
pub struct InstantiatingScript(pub ScriptCell);

#[derive(Component)]
#[require(LastFixedUpdate)]
pub struct ScriptGuest(pub Arc<ScriptInstance>);

pub fn instantiate_scripts(
    wasms: Res<Assets<Wasm>>,
    to_instantiate: Query<
        (
            Entity,
            &Script,
            &ApiPermissions,
            NameOrEntity,
            &Prim,
            &HsdChild,
        ),
        (Without<InstantiatingScript>, Without<ScriptGuest>),
    >,
    docs: Query<(&HsdDocId, &Hsd, Has<QuotaExempt>)>,
    mut commands: Commands,
) {
    for (entity, script, perms, name, prim, doc_ent) in to_instantiate {
        let Some(wasm) = wasms.get(&script.0) else {
            continue;
        };
        let Ok((doc_id, doc, exempt)) = docs.get(doc_ent.0) else {
            continue;
        };
        let quota = if exempt {
            Quota::unlimited()
        } else {
            unavi_space::quota::document_quota(doc_id.0)
        };

        let bytes = wasm.0.clone();
        let name = name.to_string();

        let runtime = Runtime {
            api: Arc::new(Api {
                state: Arc::clone(&doc.0),
                doc_id: doc_id.0,
                prim: prim.0,
                permissions: perms.clone(),
                quota,
                wired_agent: Mutex::default(),
                wired_event: Mutex::default(),
                wired_input: Mutex::default(),
                wired_kv: Mutex::default(),
                wired_scene: Mutex::default(),
                wired_wds: Mutex::default(),
            }),
        };

        let cell: ScriptCell = Arc::new(std::sync::Mutex::new(None));

        spawn_async_task({
            let cell = Arc::clone(&cell);
            async move {
                let instance = ScriptInstance::instantiate(&bytes, &name, runtime).await;
                *cell.lock().expect("mutex poisoned") = Some(instance);
            }
        });

        commands.entity(entity).insert(InstantiatingScript(cell));
    }
}

pub fn poll_instantiating(
    instantiating: Query<(Entity, &InstantiatingScript)>,
    mut commands: Commands,
) {
    for (entity, cell) in instantiating {
        let mut guard = cell.0.lock().expect("mutex poisoned");
        if let Some(instance) = guard.take() {
            commands
                .entity(entity)
                .remove::<InstantiatingScript>()
                .insert(ScriptGuest(Arc::new(instance)));
        }
    }
}
