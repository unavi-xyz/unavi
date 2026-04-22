use blake3::Hash;
use iroh_gossip::api::{Event, GossipReceiver, GossipSender};
use n0_future::StreamExt;
use tracing::{info, warn};
use wds::signed_bytes::SignedBytes;

use crate::gossip::GossipCtx;

pub async fn handle_gossip_inbound(
    ctx: &GossipCtx,
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
                // let signed_msg =
                //     match postcard::from_bytes::<SignedBytes<SpaceGossip>>(&msg.content) {
                //         Ok(v) => v,
                //         Err(err) => {
                //             warn!(?err, "got invalid gossip message");
                //             continue;
                //         }
                //     };
                //
                // let payload = match signed_msg.payload() {
                //     Ok(p) => p,
                //     Err(err) => {
                //         warn!(?err, "failed to decode gossip payload");
                //         continue;
                //     }
                // };
                //
                // // Verify signature.
                // let Ok(sig_bytes) = signed_msg.signature().try_into() else {
                //     warn!("invalid signature length: {}", signed_msg.signature().len());
                //     continue;
                // };
                // let sig = Signature::from_bytes(sig_bytes);
                //
                // if let Err(err) = payload.sender.verify(signed_msg.payload_bytes(), &sig) {
                //     warn!(?err, "invalid gossip signature");
                //     continue;
                // }
                //
                // match payload.msg {
                //     SpaceGossipMsg::Join(addr) => {
                //         if addr.id != payload.sender {
                //             warn!("join address does not match sender");
                //             continue;
                //         }
                //
                //         handle_join_broadcast(state, payload.sender, addr, space_id).await;
                //     }
                //     SpaceGossipMsg::StateDelta(delta) => {
                //         let _ = state.event_tx.try_send(NetworkEvent::PeerStateDelta {
                //             peer: delta.sender,
                //             delta,
                //         });
                //     }
                // }
            }
        }
    }

    Ok(())
}
