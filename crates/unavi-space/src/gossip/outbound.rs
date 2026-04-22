use iroh_gossip::api::GossipSender;
use wds::signed_bytes::{IrohSigner, Signable};

use crate::{
    gossip::{GossipCtx, SpaceBroadcast, SpaceMessage},
    presence::PRESENCE_INTERVAL,
};

pub async fn handle_gossip_outbound(ctx: &GossipCtx, tx: &GossipSender) -> anyhow::Result<()> {
    let signer = IrohSigner(ctx.endpoint.secret_key());

    loop {
        let broadcast = SpaceBroadcast {
            sender: ctx.endpoint.id(),
            msg: SpaceMessage::Presence,
        };

        let bytes = postcard::to_stdvec(&broadcast.sign(&signer)?)?;
        tx.broadcast(bytes.into()).await?;

        n0_future::time::sleep(PRESENCE_INTERVAL).await;
    }
}
