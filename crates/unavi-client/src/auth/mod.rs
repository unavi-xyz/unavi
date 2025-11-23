use std::{str::FromStr, sync::Arc, time::Duration};

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::TaskPool};
use dwn::{Actor, document_key::DocumentKey};
use unavi_constants::{
    REMOTE_DWN_URL, SPACE_HOST_DID, protocols::SPACE_HOST_PROTOCOL, schemas::ServerInfo,
};
use xdid::{
    core::{did::Did, did_url::DidUrl},
    methods::{
        key::{DidKeyPair, PublicKey},
        web::reqwest::Url,
    },
};

use crate::{
    LocalDwn,
    async_commands::ASYNC_COMMAND_QUEUE,
    space::{Space, record_ref_url::new_record_ref_url},
};

mod home_space;
mod key_pair;

#[derive(Event, Default)]
pub struct LoginEvent;

pub fn trigger_login(mut commands: Commands) {
    commands.trigger(LoginEvent);
}

#[derive(Resource, Default)]
/// Identity of the local user.
pub struct LocalActor(pub Option<Actor>);

pub fn handle_login(_: On<LoginEvent>, mut local_actor: ResMut<LocalActor>, dwn: Res<LocalDwn>) {
    let pair = match key_pair::get_or_create_key() {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to get or create keypair: {e:?}");
            return;
        }
    };

    let did = pair.public().to_did();
    info!("Loaded identity: {did}");

    let mut actor = Actor::new(did, dwn.0.clone());

    let key = Arc::<DocumentKey>::new(pair.into());
    actor.sign_key = Some(key.clone());
    actor.auth_key = Some(key);

    let remote_url = Url::parse(REMOTE_DWN_URL).expect("parse remote url");
    actor.remote = Some(remote_url.clone());

    local_actor.0 = Some(actor.clone());

    let pool = TaskPool::get_thread_executor();

    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let task = rt.spawn(async move {
            info!("Syncing with remote DWN: {remote_url}");

            if let Err(e) = actor.sync().await {
                error!("Failed to sync with remote DWN: {e:?}");
            }

            loop {
                // For now, join a random space (or create one if none exist).
                // Eventually we will only join the user's home world on start.
                match join_or_create_space(&actor).await {
                    Ok(()) => break,
                    Err(e) => {
                        error!("Failed to join a space: {e:?}");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();
}

async fn join_or_create_space(actor: &Actor) -> anyhow::Result<()> {
    if let Some(space_url) = find_public_space(actor).await? {
        let mut commands = CommandQueue::default();
        commands.push(bevy::ecs::system::command::spawn_batch([Space::new(
            space_url,
        )]));
        ASYNC_COMMAND_QUEUE.0.send(commands)?;
    } else {
        home_space::join_home_space(actor).await?;
    }

    Ok(())
}

async fn find_public_space(actor: &Actor) -> anyhow::Result<Option<DidUrl>> {
    let space_host = Did::from_str(SPACE_HOST_DID)?;

    // TODO: Fetch from space host
    let host_dwn = Url::parse(REMOTE_DWN_URL)?;

    let infos = actor
        .query()
        .protocol(SPACE_HOST_PROTOCOL.to_string())
        .protocol_path("space/server-info".to_string())
        .target(&space_host)
        .send(&host_dwn)
        .await?;

    info!("Found {} hosted spaces", infos.len());

    for record in infos {
        let Some(space_id) = record.entry().context_id.as_deref() else {
            continue;
        };

        let info = if let Some(data) = record.data() {
            serde_json::from_slice::<ServerInfo>(data)?
        } else {
            let Some(full_record) = actor
                .read(record.entry().record_id.clone())
                .target(&space_host)
                .send(&host_dwn)
                .await?
            else {
                continue;
            };

            let Some(data) = full_record.data() else {
                continue;
            };

            serde_json::from_slice::<ServerInfo>(data)?
        };

        info!(
            "Found space with {}/{} players",
            info.num_players, info.max_players
        );
        if info.num_players >= info.max_players {
            continue;
        }

        let space_url = new_record_ref_url(space_host, space_id);
        return Ok(Some(space_url));
    }

    Ok(None)
}
