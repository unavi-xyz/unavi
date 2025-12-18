use std::time::Duration;

use futures::StreamExt;
use iroh::{EndpointId, Signature};
use iroh_gossip::{TopicId, api::Event};
use iroh_tickets::endpoint::EndpointTicket;
use log::info;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use wired_data_store::RecordId;

use crate::networking::thread::ThreadState;

/// Presence message broadcasted on gassip topic.
/// Used for player discovery.
#[derive(Serialize, Deserialize)]
struct Presence {
    peer: PeerLocation,
    signature: Signature,
}

#[derive(Serialize, Deserialize)]
struct PeerLocation {
    space: RecordId,
    endpoint: EndpointId,
    expires: i64,
}

const PEER_BUF_SIZE: usize = core::mem::size_of::<PeerLocation>() * 2;

const PRESENCE_TTL: Duration = Duration::from_mins(1);

pub async fn handle_join(
    state: ThreadState,
    id: RecordId,
    peers: Vec<EndpointTicket>,
) -> anyhow::Result<()> {
    // TODO: Read record
    //       If SFU server specified connect to it, otherwise p2p gossip

    // Gossip for p2p connections.
    let id_bytes =
        id.0.to_bytes()
            .into_iter()
            .take(32)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid CID"))?;

    let topic_id = TopicId::from_bytes(id_bytes);

    // TODO: Where to bootstrap from?
    let bootstrap = peers.into_iter().map(|p| p.endpoint_addr().id).collect();
    info!("Bootstrap: {bootstrap:#?}");
    let mut topic = state.gossip.subscribe_and_join(topic_id, bootstrap).await?;

    let peer = PeerLocation {
        space: id,
        endpoint: state.endpoint.id(),
        expires: (OffsetDateTime::now_utc() + PRESENCE_TTL).unix_timestamp(),
    };

    let peer_bytes = postcard::to_vec::<_, PEER_BUF_SIZE>(&peer)?;
    info!("peer serialized len: {}", peer_bytes.len());
    let signature = state.endpoint.secret_key().sign(peer_bytes.as_slice());

    let presence = Presence { peer, signature };
    let presence_bytes = postcard::to_allocvec(&presence)?;
    info!("presence serialized len: {}", presence_bytes.len());

    topic.broadcast(presence_bytes.into()).await?;

    // TODO: rebroadcast presence on an interval

    while let Some(event) = topic.next().await {
        let Event::Received(message) = event? else {
            continue;
        };

        info!("incoming gossip: {message:?}");
    }

    Ok(())
}
