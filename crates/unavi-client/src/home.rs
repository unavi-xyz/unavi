use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_hsd::asset::HsdAsset;
use bevy_wds::{LocalActor, SyncTargets};
use hsd::Hsd;
use loro::LoroDoc;
use loro_surgeon::Reconcile;
use unavi_space::Space;
use unavi_util::{async_commands::ASYNC_COMMAND_QUEUE, async_task::spawn_async_task};
use wds::actor::Actor;
use wired_schemas::{SCHEMA_HOME, SCHEMA_HSD, SCHEMA_SPACE};

const DEFAULT_HOME_HSD: &str = "hsd/unavi_default_home.hsd";

#[derive(Default)]
pub struct JoinState {
    joined: bool,
    hsd: Option<Handle<HsdAsset>>,
}

pub fn join_home_space(
    asset_server: Res<AssetServer>,
    hsds: Res<Assets<HsdAsset>>,
    local_actor: Query<(&LocalActor, &SyncTargets)>,
    mut state: Local<JoinState>,
) {
    if state.joined {
        return;
    }

    let Ok((local_actor, sync_targets)) = local_actor.single() else {
        return;
    };

    let local_actor = local_actor.0.clone();
    let remote_actor = sync_targets.0.first().cloned();

    if let Some(handle) = &state.hsd {
        let Some(hsd) = hsds.get(handle) else {
            return;
        };

        let hsd = hsd.doc.clone();

        spawn_async_task(async move {
            if let Err(err) = create_and_join_home(local_actor, remote_actor, hsd).await {
                error!(?err, "Failed to join home space");
            }
        });

        state.joined = true;
    } else {
        let handle = asset_server.load(DEFAULT_HOME_HSD);
        state.hsd = Some(handle);
    }
}

async fn create_and_join_home(
    local_actor: Actor,
    remote_actor: Option<Actor>,
    hsd: Hsd,
) -> anyhow::Result<()> {
    let did = local_actor.identity().did();

    let mut actors = vec![local_actor.clone()];
    if let Some(ref remote) = remote_actor {
        actors.push(remote.clone());
    }

    let hsd_doc = LoroDoc::new();
    let map = hsd_doc.get_map("hsd");
    hsd.reconcile(&map)?;

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

    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([Space(res.id)]));
    ASYNC_COMMAND_QUEUE.0.send(commands).await?;

    Ok(())
}
