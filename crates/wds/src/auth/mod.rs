//! Challenge-response DID authentication.

use std::{fmt::Debug, sync::Arc, time::Duration};

use anyhow::bail;
use irpc::{Client, WithChannels, channel::oneshot, rpc_requests};
use irpc_iroh::IrohProtocol;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tracing::error;
use xdid::{core::did::Did, resolver::DidResolver};

use crate::{ConnectionState, auth::jwk::verify_jwk_signature, signed_bytes::SignedBytes};

mod jwk;

pub const ALPN: &[u8] = b"wds/auth";

pub fn protocol(conn: Arc<ConnectionState>) -> IrohProtocol<AuthService> {
    let (tx, mut rx) = irpc::channel::mpsc::channel(2);

    tokio::task::spawn(async move {
        while let Err(e) = handle_requests(&conn, &mut rx).await {
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
    AnswerChallenge(SignedBytes<Challenge>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    did: Did,
    nonce: Nonce,
}

#[derive(Default)]
struct HandlerState {
    nonces: scc::HashMap<Nonce, Did>,
}

async fn handle_requests(
    conn: &Arc<ConnectionState>,
    rx: &mut irpc::channel::mpsc::Receiver<AuthMessage>,
) -> anyhow::Result<()> {
    let state = Arc::new(HandlerState::default());

    while let Some(msg) = rx.recv().await? {
        let conn = Arc::clone(conn);
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            if let Err(e) = handle_message(conn, state, msg).await {
                error!("Error handling message: {e:?}");
            }
        });
    }

    Ok(())
}

const MAX_NONCES: usize = 16;
const NONCE_TTL: Duration = Duration::from_mins(5);

async fn handle_message(
    conn: Arc<ConnectionState>,
    state: Arc<HandlerState>,
    msg: AuthMessage,
) -> anyhow::Result<()> {
    match msg {
        AuthMessage::RequestChallenge(WithChannels { inner, tx, .. }) => {
            if state.nonces.len() > MAX_NONCES {
                return Ok(());
            }

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
            let challenge = inner.0.payload()?;

            let Some(did) = state
                .nonces
                .read_async(&challenge.nonce, |_, d| d.clone())
                .await
            else {
                // Invalid nonce.
                tx.send(false).await?;
                return Ok(());
            };

            if did != challenge.did {
                // Invalid DID.
                tx.send(false).await?;
                return Ok(());
            }

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

                let is_valid =
                    verify_jwk_signature(jwk, inner.0.signature(), inner.0.payload_bytes());

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

            if conn.authentication.set(did).is_err() {
                // Already authenticated.
                tx.send(false).await?;
                return Ok(());
            }

            tx.send(true).await?;
        }
    }

    Ok(())
}
