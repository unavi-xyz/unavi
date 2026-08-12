use std::sync::Arc;

use iroh_docs::NamespaceId;
use irpc::{
    Client,
    channel::oneshot,
    rpc_requests,
};
use irpc_iroh::IrohProtocol;
use serde::{
    Deserialize,
    Serialize,
};
use tracing::error;
use wds::{
    SessionToken,
    signed_bytes::SignedBytes,
};

use crate::{
    RegistryContext,
    entry::{
        Presence,
        Submission,
    },
    error::RegistryError,
    views::ViewIds,
};

mod presence;
mod submit;

pub const ALPN: &[u8] = b"wired/registry";

pub fn protocol(
    ctx: Arc<RegistryContext>,
) -> (Client<RegistryService>, IrohProtocol<RegistryService>) {
    let (tx, mut rx) = irpc::channel::mpsc::channel(16);

    n0_future::task::spawn(async move {
        while let Some(msg) = match rx.recv().await {
            Ok(msg) => msg,
            Err(err) => {
                error!("registry receive failed: {err:?}");
                None
            }
        } {
            let ctx = Arc::clone(&ctx);
            n0_future::task::spawn(async move {
                if let Err(err) = handle_message(ctx, msg).await {
                    error!("registry request failed: {err:?}");
                }
            });
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    (client, IrohProtocol::with_sender(local_sender))
}

#[rpc_requests(message = RegistryMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RegistryService {
    /// Publish a durable catalog entry.
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Submit)]
    Submit {
        s:          SessionToken,
        submission: SignedBytes<Submission>,
    },
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Retract)]
    Retract { s: SessionToken, ns: NamespaceId },
    /// Heartbeat live occupancy. Never persisted.
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Announce)]
    Announce {
        s:        SessionToken,
        presence: SignedBytes<Presence>,
    },
    /// Current occupants of a namespace, each individually signed.
    #[rpc(tx=oneshot::Sender<Result<Vec<SignedBytes<Presence>>, RegistryError>>)]
    #[wrap(Occupants)]
    Occupants { ns: NamespaceId },
    /// The namespaces of this registry's view docs. Unauthenticated: reading a
    /// registry requires no account.
    #[rpc(tx=oneshot::Sender<Result<ViewIds, RegistryError>>)]
    #[wrap(Views)]
    Views,
}

macro_rules! authenticate {
    ($ctx:tt, $inner:tt, $tx:tt) => {
        match $ctx.session_did(&$inner.s).await {
            Some(did) => did,
            None => {
                $tx.send(Err($crate::error::RegistryError::Unauthenticated))
                    .await?;
                return Ok(());
            }
        }
    };
}

pub(crate) use authenticate;

async fn handle_message(ctx: Arc<RegistryContext>, msg: RegistryMessage) -> anyhow::Result<()> {
    match msg {
        RegistryMessage::Submit(channels) => submit::submit(ctx, channels).await,
        RegistryMessage::Retract(channels) => submit::retract(ctx, channels).await,
        RegistryMessage::Announce(channels) => presence::announce(ctx, channels).await,
        RegistryMessage::Occupants(channels) => presence::occupants(ctx, channels).await,
        RegistryMessage::Views(channels) => {
            let irpc::WithChannels { tx, .. } = channels;
            tx.send(Ok(ctx.views.ids())).await?;
            Ok(())
        }
    }
}
