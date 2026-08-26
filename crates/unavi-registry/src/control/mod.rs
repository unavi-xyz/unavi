use std::sync::Arc;

use iroh_docs::NamespaceId;
use irpc::{
    channel::oneshot,
    rpc_requests,
};
use serde::{
    Deserialize,
    Serialize,
};
use unavi_identity::signed_bytes::SignedBytes;
use xdid::core::did::Did;

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
pub mod protocol;
mod submit;

pub const ALPN: &[u8] = b"wired/registry";

#[rpc_requests(message = RegistryMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RegistryService {
    /// Publish a durable catalog entry.
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Submit)]
    Submit { submission: SignedBytes<Submission> },
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Retract)]
    Retract { ns: NamespaceId },
    /// Heartbeat live occupancy. Never persisted.
    #[rpc(tx=oneshot::Sender<Result<(), RegistryError>>)]
    #[wrap(Announce)]
    Announce { presence: SignedBytes<Presence> },
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

/// The DID this call was made under, refusing the call outright if the
/// connection proved none.
macro_rules! caller {
    ($caller:tt, $tx:tt) => {
        match $caller {
            Some(did) => did,
            None => {
                $tx.send(Err($crate::error::RegistryError::Unauthenticated))
                    .await?;
                return Ok(());
            }
        }
    };
}

pub(crate) use caller;

/// Runs one call. `caller` is the DID the connection proved, or `None` for a
/// peer that proved nothing: the read side serves those, and only the write
/// side refuses them.
pub(crate) async fn handle_message(
    ctx: Arc<RegistryContext>,
    caller: Option<Did>,
    msg: RegistryMessage,
) -> anyhow::Result<()> {
    match msg {
        RegistryMessage::Submit(channels) => submit::submit(ctx, caller, channels).await,
        RegistryMessage::Retract(channels) => submit::retract(ctx, caller, channels).await,
        RegistryMessage::Announce(channels) => presence::announce(ctx, caller, channels).await,
        RegistryMessage::Occupants(channels) => presence::occupants(ctx, channels).await,
        RegistryMessage::Views(channels) => {
            let irpc::WithChannels { tx, .. } = channels;
            tx.send(Ok(ctx.views.ids())).await?;
            Ok(())
        }
    }
}
