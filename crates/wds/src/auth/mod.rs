//! Challenge-response DID authentication.

use std::{sync::Arc, time::Duration};

use anyhow::bail;
use irpc::{Client, WithChannels, channel::oneshot, rpc_requests};
use irpc_iroh::IrohProtocol;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tracing::error;
use xdid::{core::did::Did, resolver::DidResolver};

use crate::auth::jwk::verify_jwk_signature;

mod jwk;

pub const ALPN: &[u8] = b"wds/auth";

pub fn protocol() -> IrohProtocol<AuthService> {
    let (tx, mut rx) = irpc::channel::mpsc::channel(2);

    tokio::task::spawn(async move {
        while let Err(e) = handle_requests(&mut rx).await {
            error!("Error handling request: {e:?}");
        }
    });

    let client = Client::local(tx);
    let local_sender = client.as_local().expect("local client");

    IrohProtocol::with_sender(local_sender)
}

type Nonce = [u8; 32];

#[rpc_requests(message = AuthMessage)]
#[derive(Debug, Serialize, Deserialize)]
pub enum AuthService {
    #[rpc(tx=oneshot::Sender<Nonce>)]
    #[wrap(RequestChallenge)]
    RequestChallenge(Did),
    #[rpc(tx=oneshot::Sender<bool>)]
    #[wrap(AnswerChallenge)]
    AnswerChallenge {
        payload: Payload,
        signature: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    did: Did,
    nonce: Nonce,
}

const PAYLOAD_SIZE: usize = size_of::<Payload>();

#[derive(Default)]
struct HandlerState {
    nonces: scc::HashMap<Nonce, Did>,
}

async fn handle_requests(
    rx: &mut irpc::channel::mpsc::Receiver<AuthMessage>,
) -> anyhow::Result<()> {
    let state = Arc::new(HandlerState::default());

    while let Some(msg) = rx.recv().await? {
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = handle_message(state, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

const NONCE_TTL: Duration = Duration::from_mins(5);

async fn handle_message(state: Arc<HandlerState>, msg: AuthMessage) -> anyhow::Result<()> {
    match msg {
        AuthMessage::RequestChallenge(WithChannels { inner, tx, .. }) => {
            let mut nonce = Nonce::default();
            rand::rng().fill_bytes(&mut nonce);

            if let Err((_, did)) = state.nonces.insert_async(nonce, inner.0).await {
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
            let Some(did) = state
                .nonces
                .read_async(&inner.payload.nonce, |_, d| d.clone())
                .await
            else {
                // Invalid nonce.
                tx.send(false).await?;
                return Ok(());
            };

            if did != inner.payload.did {
                // Invalid DID.
                tx.send(false).await?;
                return Ok(());
            }

            let payload_bytes = postcard::to_vec::<_, PAYLOAD_SIZE>(&inner.payload)?;

            // Resolve DID authentication keys.
            let resolver = DidResolver::new()?;
            let doc = resolver.resolve(&did).await?;

            let Some(auth_methods) = &doc.authentication else {
                tx.send(false).await?;
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

                let is_valid = verify_jwk_signature(jwk, &inner.signature, &payload_bytes);

                if is_valid {
                    found_valid = true;
                    break;
                }
            }

            if !found_valid {
                // Invalid signature.
                tx.send(false).await?;
                return Ok(());
            }

            // TODO mark connection as authenticated as DID
        }
    }

    Ok(())
}
