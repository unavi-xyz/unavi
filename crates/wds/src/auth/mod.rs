//! ## Auth Protocol
//!
//! Challenge-response DID authentication.

use std::{fmt::Debug, sync::Arc, time::Duration};

use anyhow::bail;
use iroh::EndpointId;
use irpc::{Client, WithChannels, channel::oneshot, rpc_requests};
use irpc_iroh::IrohProtocol;
use rand::RngCore;
use scc::HashCache;
use serde::{Deserialize, Serialize};
use tracing::error;
use xdid::{core::did::Did, resolver::DidResolver};

use crate::{
    ConnectionState, SessionToken, StoreContext,
    auth::jwk::verify_jwk_signature,
    signed_bytes::{Signable, SignedBytes},
};

mod jwk;

pub const ALPN: &[u8] = b"wds/auth";

pub fn protocol(ctx: Arc<StoreContext>) -> (Client<AuthService>, IrohProtocol<AuthService>) {
    let (tx, mut rx) = irpc::channel::mpsc::channel(16);

    tokio::task::spawn(async move {
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

        tokio::spawn(async move {
            if let Err(e) = handle_message(ctx, state, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

const NONCE_TTL: Duration = Duration::from_mins(3);

async fn handle_message(
    ctx: Arc<StoreContext>,
    state: Arc<HandlerState>,
    msg: AuthMessage,
) -> anyhow::Result<()> {
    match msg {
        AuthMessage::RequestChallenge(WithChannels { inner, tx, .. }) => {
            let mut nonce = Nonce::default();
            rand::rng().fill_bytes(&mut nonce);

            if let Err((_, did)) = state.nonces.put_async(nonce, inner.0).await {
                bail!("Failed to generate nonce for {did}")
            }

            // Remove nonce after time limit.
            tokio::spawn(async move {
                tokio::time::sleep(NONCE_TTL).await;
                state.nonces.remove_async(&nonce).await;
            });

            tx.send(nonce).await?;
        }
        AuthMessage::AnswerChallenge(WithChannels { inner, tx, .. }) => {
            let challenge = inner.0.payload()?;

            if challenge.host != ctx.endpoint.id() {
                // Invalid host.
                tx.send(None).await?;
                return Ok(());
            }

            let Some(did) = state
                .nonces
                .read_async(&challenge.nonce, |_, d| d.clone())
                .await
            else {
                // Invalid nonce.
                tx.send(None).await?;
                return Ok(());
            };

            if did != challenge.did {
                // Invalid DID.
                tx.send(None).await?;
                return Ok(());
            }

            // Resolve DID authentication keys.
            let resolver = DidResolver::new()?;
            let doc = resolver.resolve(&did).await?;

            let Some(auth_methods) = &doc.authentication else {
                tx.send(None).await?;
                return Ok(());
            };

            // Validate signature.
            let mut found_valid = false;

            for method in auth_methods {
                let Some(map) = doc.resolve_verification_method(method) else {
                    continue;
                };

                let Some(jwk) = &map.public_key_jwk else {
                    continue;
                };

                let is_valid =
                    verify_jwk_signature(jwk, inner.0.signature(), inner.0.payload_bytes());

                if is_valid {
                    found_valid = true;
                    break;
                }
            }

            if !found_valid {
                // Invalid signature.
                tx.send(None).await?;
                return Ok(());
            }

            let mut token = SessionToken::default();
            rand::rng().fill_bytes(&mut token);

            if ctx
                .connections
                .insert_async(token, ConnectionState { did })
                .await
                .is_err()
            {
                tx.send(None).await?;
                return Ok(());
            }

            tx.send(Some(token)).await?;
        }
    }

    Ok(())
}
