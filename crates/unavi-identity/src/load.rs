use std::{sync::Arc, time::Duration};

use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_iroh::{IrohEndpoint, RouterBuilderFn, RouterBuilderFnTarget};
use bevy_wds::{LocalActor, LocalBlobs};
use iroh::Endpoint;
use unavi_util::async_commands::ASYNC_COMMAND_QUEUE;
use wds::{DataStore, Identity};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

pub fn spawn_actors(trigger: On<Add, IrohEndpoint>, endpoints: Query<&IrohEndpoint>) {
    let entity = trigger.entity;

    let endpoint = endpoints
        .get(entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    unavi_wasm_compat::spawn_thread(async move {
        let mut delay_secs = 4;

        loop {
            if let Err(err) = load_store(endpoint.clone(), entity).await {
                error!(?err, "failed to load data store");
                n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                delay_secs = delay_secs.wrapping_mul(2);
                continue;
            }

            break;
        }
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
    let actor = store.local_actor(identity);

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
