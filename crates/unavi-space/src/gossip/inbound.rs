use blake3::Hash;
use iroh::Signature;
use iroh_gossip::api::{Event, GossipReceiver};
use n0_future::StreamExt;
use tracing::{info, warn};
use wds::signed_bytes::SignedBytes;

use crate::{
    gossip::{GossipCtx, SpaceBroadcast, SpaceMessage},
    presence::{PRESENCE_QUEUE, PresenceUpdate},
};

pub async fn handle_gossip_inbound(
    _ctx: &GossipCtx,
    rx: &mut GossipReceiver,
    space: Hash,
) -> anyhow::Result<()> {
    while let Some(event) = rx.next().await {
        match event? {
            Event::NeighborUp(n) => {
                info!("+neighbor: {n}");
            }
            Event::NeighborDown(n) => {
                info!("-neighbor: {n}");
            }
            Event::Lagged => warn!("lagged"),
            Event::Received(msg) => {
                let signed_bytes =
                    match postcard::from_bytes::<SignedBytes<SpaceBroadcast>>(&msg.content) {
                        Ok(v) => v,
                        Err(err) => {
                            warn!(?err, "Got invalid gossip message");
                            continue;
                        }
                    };

                let broadcast = match signed_bytes.payload() {
                    Ok(v) => v,
                    Err(err) => {
                        warn!(?err, "Failed to decode gossip payload");
                        continue;
                    }
                };

                // Verify signature.
                let Ok(sig_bytes) = signed_bytes.signature().try_into() else {
                    warn!(
                        "Invalid signature length: {}",
                        signed_bytes.signature().len()
                    );
                    continue;
                };
                let sig = Signature::from_bytes(sig_bytes);

                if let Err(err) = broadcast.sender.verify(signed_bytes.payload_bytes(), &sig) {
                    warn!(?err, "Invalid gossip signature");
                    continue;
                }

                // TODO create a "disconnect" message variant to clear presence

                // Handle message.
                match broadcast.msg {
                    SpaceMessage::Presence(peer) => {
                        if peer.id != broadcast.sender {
                            warn!("Presence address does not match sender");
                            continue;
                        }

                        PRESENCE_QUEUE
                            .0
                            .send(PresenceUpdate { peer, space })
                            .await?;
                    }
                    SpaceMessage::Unknown(i) => {
                        warn!("got unknown gossip variant: {i}");
                    }
                }
            }
        }
    }

    Ok(())
}
