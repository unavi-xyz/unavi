use std::time::Duration;

use iroh::Watcher;
use iroh_docs::NamespaceId;
use iroh_gossip::api::GossipSender;
use tokio::sync::Notify;
use tracing::info;
use unavi_identity::signed_bytes::{
    IrohSigner,
    Signable,
};

use crate::{
    gossip::{
        GossipCtx,
        SpaceBroadcast,
        SpaceMessage,
        active_changed,
        active_space,
    },
    peer::presence::PRESENCE_INTERVAL,
};

pub async fn handle_gossip_outbound(
    ctx: &GossipCtx,
    tx: &GossipSender,
    space: NamespaceId,
    wake: &Notify,
) -> anyhow::Result<()> {
    let signer = IrohSigner(ctx.endpoint.secret_key());
    let mut watcher = ctx.endpoint.watch_addr();

    let _ = n0_future::time::timeout(Duration::from_secs(15), ctx.endpoint.online()).await;

    loop {
        // Only advertise the occupied space; peers in other loaded spaces are
        // still reached via their own broadcasts, which are all received.
        if active_space() == Some(space) {
            let addr = watcher.get();

            info!("Broadcasting presence: {:?}", addr);
            let broadcast = SpaceBroadcast {
                sender: ctx.endpoint.id(),
                msg:    SpaceMessage::Presence(addr),
            };

            let bytes = postcard::to_stdvec(&broadcast.sign(&signer)?)?;
            tx.broadcast(bytes.into()).await?;
        }

        // Re-broadcast on the interval, immediately when a neighbor joins the
        // topic, or as soon as this space becomes active, so neither entering
        // a space nor a peer arriving waits out a full interval.
        tokio::select! {
            () = n0_future::time::sleep(PRESENCE_INTERVAL) => {}
            () = wake.notified() => {}
            () = active_changed().notified() => {}
        }
    }
}
