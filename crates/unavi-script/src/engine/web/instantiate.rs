use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{HsdChild, HsdRecordId, NodeId, ScriptNode};
use unavi_util::async_task::spawn_async_task;

use crate::{
    Script,
    load::asset::Wasm,
    permissions::ApiPermissions,
    runtime::{Runtime, shared::Api},
};

#[derive(Component)]
pub struct InstantiatingScript;

#[derive(Component)]
pub struct ScriptGuest;

pub fn instantiate_scripts(
    wasms: Res<Assets<Wasm>>,
    to_instantiate: Query<
        (Entity, &Script, &ApiPermissions, NameOrEntity, &ScriptNode),
        (Without<InstantiatingScript>, Without<ScriptGuest>),
    >,
    nodes: Query<(&NodeId, &HsdChild)>,
    docs: Query<&HsdRecordId>,
    mut commands: Commands,
) {
    for (entity, script, perms, name, node_ent) in to_instantiate {
        let Some(wasm) = wasms.get(&script.0) else {
            continue;
        };
        let Ok((node_id, hsd)) = nodes.get(node_ent.0) else {
            continue;
        };
        let Ok(doc_id) = docs.get(hsd.0) else {
            continue;
        };

        let bytes = wasm.0.clone();
        let name = name.to_string();

        let runtime = Runtime {
            api: Arc::new(Api {
                document: doc_id.0,
                node: node_id.0,
                permissions: perms.clone(),
                wired_input: Default::default(),
                wired_scene: Default::default(),
            }),
        };

        spawn_async_task(async move {
            crate::runtime::web::build_script(&bytes, &name, runtime).await;
        });

        commands.entity(entity).insert(InstantiatingScript);
    }
}
