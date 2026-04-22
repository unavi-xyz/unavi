use std::{sync::Arc, time::Duration};

use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{RouterBuilderFn, RouterBuilderFnTarget},
};
use bevy_wds::{LocalActor, LocalBlobs};
use iroh::Endpoint;
use unavi_util::{async_commands::ASYNC_COMMAND_QUEUE, async_task::spawn_async_task};
use wds::{DataStore, Identity};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

pub fn spawn_actors(trigger: On<Add, IrohEndpoint>, endpoints: Query<&IrohEndpoint>) {
    let entity = trigger.entity;

    let endpoint = endpoints
        .get(entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    spawn_async_task(async move {
        let mut delay_secs = 4;

        loop {
            if let Err(err) = load_store(endpoint.clone(), entity).await {
                error!(?err, "Failed to load data store");
                n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                delay_secs = delay_secs.wrapping_mul(2);
                continue;
            }
            break;
        }

        // Keep data store alive.
        // TODO merge with shared thread?
        std::future::pending::<()>().await;
    });
}

async fn load_store(endpoint: Endpoint, entity: Entity) -> anyhow::Result<()> {
    // TODO load identity from disk / browser storage
    let signing_key = P256KeyPair::generate();
    let did = signing_key.public().to_did();
    let identity = Arc::new(Identity::new(did, signing_key));

    let (store, f) = DataStore::builder(endpoint)
        .gc_timer(Duration::from_mins(15))
        .build()
        .await?;

    store.set_user_identity(Arc::clone(&identity));
    let actor = store.local_actor(identity);

    // TODO load sync targets from env

    let mut commands = CommandQueue::default();
    commands.push(bevy::ecs::system::command::spawn_batch([(
        RouterBuilderFnTarget(entity),
        RouterBuilderFn(Some(f)),
    )]));
    commands.push(bevy::ecs::system::command::spawn_batch([(
        LocalActor(actor),
        LocalBlobs(store.blobs().blobs().clone()),
    )]));
    ASYNC_COMMAND_QUEUE.0.send(commands).await?;

    Ok(())
}
