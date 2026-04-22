use std::time::Duration;

use iroh::Watcher;
use iroh_gossip::api::GossipSender;
use tracing::info;
use wds::signed_bytes::{IrohSigner, Signable};

use crate::{
    gossip::{GossipCtx, SpaceBroadcast, SpaceMessage},
    presence::PRESENCE_INTERVAL,
};

pub async fn handle_gossip_outbound(ctx: &GossipCtx, tx: &GossipSender) -> anyhow::Result<()> {
    let signer = IrohSigner(ctx.endpoint.secret_key());
    let mut watcher = ctx.endpoint.watch_addr();

    // Wait for endpoint to be online.
    let _ = n0_future::time::timeout(Duration::from_secs(15), ctx.endpoint.online()).await;

    loop {
        let addr = watcher.get();

        info!("Broadcasting presence: {:?}", addr);
        let broadcast = SpaceBroadcast {
            sender: ctx.endpoint.id(),
            msg: SpaceMessage::Presence(addr),
        };

        let bytes = postcard::to_stdvec(&broadcast.sign(&signer)?)?;
        tx.broadcast(bytes.into()).await?;

        n0_future::time::sleep(PRESENCE_INTERVAL).await;
    }
}
