use std::sync::Arc;

use bevy::{prelude::*, tasks::TaskPool};
use dwn::{Actor, document_key::DocumentKey};
use unavi_constants::REMOTE_DWN_URL;
use xdid::methods::{
    key::{DidKeyPair, PublicKey},
    web::reqwest::Url,
};

use crate::LocalDwn;

mod home_space;
mod key_pair;
mod record_ref_url;

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

            if let Err(e) = home_space::join_home_space(actor).await {
                error!("Failed to join home space: {e:?}");
            }
        });

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();
}
