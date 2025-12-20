use std::time::Duration;

use bevy::log::{error, info};
use futures::StreamExt;
use iroh::{PublicKey, Signature};
use iroh_gossip::{
    TopicId,
    api::{Event, GossipSender},
};
use serde::{Deserialize, Serialize};
use wired_data_store::RecordId;

use crate::networking::thread::ThreadState;

#[derive(Serialize, Deserialize, Debug)]
struct Beacon {
    location: BeaconContent,
    signature: Signature,
}

#[derive(Serialize, Deserialize, Debug)]
struct BeaconContent {
    space: RecordId,
    peer: PublicKey,
}

const BEACON_BUF_SIZE: usize = core::mem::size_of::<Beacon>();

pub async fn handle_space_discovery(
    bootstrap: Vec<PublicKey>,
    state: ThreadState,
    beacon_rx: &mut tokio::sync::mpsc::Receiver<RecordId>,
) -> anyhow::Result<()> {
    let topic_hash = blake3::hash(b"wired-protocol/space");
    let topic_id = TopicId::from_bytes(*topic_hash.as_bytes());

    let (topic_tx, mut topic_rx) = state
        .gossip
        .subscribe_and_join(topic_id, bootstrap.clone())
        .await?
        .split();

    while let Some(event) = topic_rx.next().await {
        let Event::Received(message) = event? else {
            continue;
        };

        if message.content.len() > BEACON_BUF_SIZE {
            continue;
        }

        let content = message.content.to_vec();
        let Ok(beacon) = postcard::from_bytes::<Beacon>(&content) else {
            continue;
        };

        info!("Got gossip beacon: {:?}", beacon);
    }

    while let Err(e) = handle_beacon_publish(&state, beacon_rx, &topic_tx).await {
        error!("Error handling space discovery: {e:?}");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    Ok(())
}

async fn handle_beacon_publish(
    state: &ThreadState,
    beacon_rx: &mut tokio::sync::mpsc::Receiver<RecordId>,
    topic_tx: &GossipSender,
) -> anyhow::Result<()> {
    while let Some(space) = beacon_rx.recv().await {
        let location = BeaconContent {
            space,
            peer: state.endpoint.id(),
        };
        let location_bytes = postcard::to_stdvec(&location)?;

        let beacon = Beacon {
            location,
            signature: state.endpoint.secret_key().sign(&location_bytes),
        };
        let beacon_bytes = postcard::to_stdvec(&beacon)?;

        let topic_tx = topic_tx.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = topic_tx.broadcast(beacon_bytes.clone().into()).await {
                    error!("Error broadcasting beacon: {e:?}");
                }

                tokio::time::sleep(Duration::from_mins(1)).await;
            }
        });
    }

    Ok(())
}
