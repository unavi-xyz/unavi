use std::time::Duration;

use blake3::Hash;
use futures::StreamExt;
use iroh::{EndpointId, PublicKey, Signature};
use iroh_gossip::{TopicId, api::Event};
use log::info;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::networking::thread::ThreadState;

/// Presence message broadcasted on gassip topic.
/// Used for player discovery.
#[derive(Serialize, Deserialize)]
struct Presence {
    peer: PeerLocation,
    signature: Signature,
}

#[derive(Serialize, Deserialize, Debug)]
struct PeerLocation {
    space: Hash,
    endpoint: EndpointId,
    expires: i64,
}

const PEER_BUF_SIZE: usize = core::mem::size_of::<PeerLocation>();
const PRESENCE_TTL: Duration = Duration::from_mins(1);

pub async fn handle_join(
    state: ThreadState,
    id: Hash,
    bootstrap: Vec<PublicKey>,
) -> anyhow::Result<()> {
    // TODO: Read record
    //       If SFU server specified connect to it, otherwise p2p gossip

    // Gossip for p2p connections.
    let topic_id = TopicId::from_bytes(*id.as_bytes());

    let mut topic = state.gossip.subscribe_and_join(topic_id, bootstrap).await?;
    info!("Joined topic: {topic_id}");

    let local_presence_bytes = {
        let peer = PeerLocation {
            space: id,
            endpoint: state.endpoint.id(),
            expires: (OffsetDateTime::now_utc() + PRESENCE_TTL).unix_timestamp(),
        };
        let peer_bytes = postcard::to_vec::<_, PEER_BUF_SIZE>(&peer)?;
        let signature = state.endpoint.secret_key().sign(peer_bytes.as_slice());
        let local_presence = Presence { peer, signature };
        postcard::to_allocvec(&local_presence)?
    };

    topic.broadcast(local_presence_bytes.clone().into()).await?;

    // TODO: rebroadcast presence on an interval

    while let Some(event) = topic.next().await {
        match event? {
            Event::NeighborUp(id) => {
                info!("Neighbor up: {id}");
                topic.broadcast(local_presence_bytes.clone().into()).await?;
            }
            Event::NeighborDown(id) => {
                info!("Neighbor down: {id}");
            }
            Event::Received(message) => {
                if message.content.len() > PEER_BUF_SIZE {
                    continue;
                }

                let content = message.content.to_vec();
                let Ok(presence) = postcard::from_bytes::<Presence>(&content) else {
                    // Not a presence message.
                    continue;
                };

                info!("Got gossip presence: {:?}", presence.peer);
            }
            Event::Lagged => {}
        }
    }

    Ok(())
}
