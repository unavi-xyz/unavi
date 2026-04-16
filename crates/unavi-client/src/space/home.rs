use std::path::PathBuf;

use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_wds::{LocalActor, SyncTargets};
use wds::actor::Actor;
use wired_schemas::{SCHEMA_HOME, SCHEMA_HSD, SCHEMA_SPACE};

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    space::{Space, SpaceDoc, lifecycle::JoinedSpace},
};

const DEFAULT_HOME_HSD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/hsd/unavi_default_home.hsd"
);

#[derive(Default)]
pub struct JoinState {
    ready: bool,
    joined: bool,
}

pub fn join_home_space(
    local_actor: Query<(&LocalActor, &SyncTargets)>,
    mut state: Local<JoinState>,
) {
    if state.joined {
        return;
    }

    let Ok((local_actor, sync_targets)) = local_actor.single() else {
        return;
    };

    // Wait one frame after LocalActor spawns so recv_network_event
    // can drain any pending remote actors into SyncTargets.
    if !state.ready {
        state.ready = true;
        return;
    }

    let local_actor = local_actor.0.clone();
    let remote_actor = sync_targets.0.first().cloned();

    unavi_wasm_compat::spawn_thread(async move {
        if let Err(err) = create_and_join_home(local_actor, remote_actor).await {
            error!(?err, "Failed to join home space");
        }
    });

    state.joined = true;
}

async fn create_and_join_home(
    local_actor: Actor,
    remote_actor: Option<Actor>,
) -> anyhow::Result<()> {
    let did = local_actor.identity().did();

    let mut actors = vec![local_actor.clone()];
    if let Some(ref remote) = remote_actor {
        actors.push(remote.clone());
    }

    let hsd_doc =
        bevy_hsd::load_hsd::build_hsd_doc_from_file(PathBuf::from(DEFAULT_HOME_HSD), &actors)
            .await?;

    let hsd_snapshot = hsd_doc
        .export(loro::ExportMode::Snapshot)
        .map_err(|e| anyhow::anyhow!("export hsd snapshot: {e:?}"))?;

    let res = local_actor
        .create_record()
        .public()
        .add_schema("home", &*SCHEMA_HOME, |_| Ok(()))?
        .add_schema("space", &*SCHEMA_SPACE, |doc| {
            let map = doc.get_map("space");
            map.insert("name", format!("{did}'s Home"))?;
            Ok(())
        })?
        .add_schema("hsd", &*SCHEMA_HSD, |doc| {
            doc.import(&hsd_snapshot)
                .map_err(|e| anyhow::anyhow!("import hsd snapshot: {e:?}"))?;
            Ok(())
        })?
        .sync_to(remote_actor.clone())
        .send()
        .await?;

    info!(id = %res.id, "Created home space");

    // Spawn space entity declaratively — the JoinedSpace marker drives
    // the networking thread to join via on_space_joined.
    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([(
        Space(res.id),
        SpaceDoc(res.doc),
        JoinedSpace,
    )]));
    ASYNC_COMMAND_QUEUE.0.send(commands).await?;

    Ok(())
}
