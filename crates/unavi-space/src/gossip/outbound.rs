use std::time::Duration;

use iroh_gossip::api::GossipSender;
use wds::signed_bytes::{IrohSigner, Signable};

use crate::gossip::{GossipCtx, SpaceBroadcast, SpaceMessage};

const PRESENCE_INTERVAL: Duration = Duration::from_secs(15);

pub async fn handle_gossip_outbound(ctx: &GossipCtx, tx: &GossipSender) -> anyhow::Result<()> {
    loop {
        let broadcast = SpaceBroadcast {
            sender: ctx.endpoint.id(),
            msg: SpaceMessage::Presence.sign(&IrohSigner(ctx.endpoint.secret_key()))?,
        };

        let bytes = postcard::to_stdvec(&broadcast)?;
        tx.broadcast(bytes.into()).await?;

        n0_future::time::sleep(PRESENCE_INTERVAL).await;
    }
}
