//! ## Auth Protocol
//!
//! Challenge-response DID authentication.
//!
//! Explicitly for WDS operations. Should not be used as general purpose
//! connection-level auth.

use std::{fmt::Debug, sync::Arc};

use iroh::EndpointId;
use irpc::{Client, channel::oneshot, rpc_requests};
use irpc_iroh::IrohProtocol;
use scc::HashCache;
use serde::{Deserialize, Serialize};
use tracing::error;
use xdid::core::did::Did;

use crate::{
    SessionToken, StoreContext,
    signed_bytes::{Signable, SignedBytes},
};

pub mod client;
pub mod jwk;
mod server;

pub const ALPN: &[u8] = b"wds/auth";

pub fn protocol(ctx: Arc<StoreContext>) -> (Client<AuthService>, IrohProtocol<AuthService>) {
    let (tx, mut rx) = irpc::channel::mpsc::channel(16);

    unavi_wasm_compat::spawn(async move {
        while let Err(e) = handle_requests(&ctx, &mut rx).await {
            error!("Error handling request: {e:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    (client, IrohProtocol::with_sender(local_sender))
}

type Nonce = [u8; 32];

#[rpc_requests(message = AuthMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthService {
    #[rpc(tx=oneshot::Sender<Nonce>)]
    #[wrap(RequestChallenge)]
    RequestChallenge(Did),
    /// Answer a challenge, signing the nonce with either an:
    /// - Authentiaction VC
    /// - WDS service endpoint key
    #[rpc(tx=oneshot::Sender<Option<SessionToken>>)]
    #[wrap(AnswerChallenge)]
    AnswerChallenge(SignedBytes<Challenge>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    /// Key must verify they are authenticating as DID.
    pub did: Did,
    /// Key must verify they are authenticating to us.
    /// Prevents impersonation by forwarding signed challenge to another node.
    pub host: EndpointId,
    /// Key must sign our given nonce.
    pub nonce: Nonce,
    // TODO: add timestamp + expiration
}

impl Signable for Challenge {}

struct HandlerState {
    nonces: scc::HashCache<Nonce, Did>,
}

async fn handle_requests(
    ctx: &Arc<StoreContext>,
    rx: &mut irpc::channel::mpsc::Receiver<AuthMessage>,
) -> anyhow::Result<()> {
    let state = Arc::new(HandlerState {
        nonces: HashCache::with_capacity(32, 2048),
    });

    while let Some(msg) = rx.recv().await? {
        let ctx = Arc::clone(ctx);
        let state = Arc::clone(&state);

        unavi_wasm_compat::spawn(async move {
            if let Err(e) = server::handle_message(ctx, state, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}
