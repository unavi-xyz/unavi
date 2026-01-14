use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::bail;
use iroh::{EndpointId, Signature};
use irpc::WithChannels;
use rand::RngCore;
use tracing::debug;
use xdid::resolver::DidResolver;

use crate::{
    ConnectionState, SessionToken, StoreContext,
    auth::{AuthMessage, HandlerState, Nonce, jwk::verify_jwk_signature},
};

const NONCE_TTL: Duration = Duration::from_mins(3);

pub async fn handle_message(
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
            n0_future::task::spawn(async move {
                n0_future::time::sleep(NONCE_TTL).await;
                state.nonces.remove_async(&nonce).await;
            });

            tx.send(nonce).await?;
        }
        AuthMessage::AnswerChallenge(WithChannels { inner, tx, .. }) => {
            let challenge = inner.0.payload()?;

            if challenge.host != ctx.endpoint.id() {
                debug!("invalid host");
                tx.send(None).await?;
                return Ok(());
            }

            let Some(did) = state
                .nonces
                .read_async(&challenge.nonce, |_, d| d.clone())
                .await
            else {
                debug!("invalid nonce");
                tx.send(None).await?;
                return Ok(());
            };

            if did != challenge.did {
                debug!("wrong did");
                tx.send(None).await?;
                return Ok(());
            }

            let resolver = DidResolver::new()?;
            let doc = resolver.resolve(&did).await?;

            // Validate signature.
            let mut found_valid = false;

            // Check auth methods.
            if let Some(auth_methods) = &doc.authentication {
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
            }

            // Check WDS services.
            // We allow defined WDSes to authenticate on behalf of the DID.
            // This enables cross-WDS operations like reading or syncing.
            // Any written data still must be signed and verified by an attestation method.
            if !found_valid && let Some(services) = doc.service {
                for service in services {
                    if service.id != "wds" {
                        continue;
                    }

                    for t in service.typ {
                        let Ok(endpoint) = EndpointId::from_str(&t) else {
                            continue;
                        };

                        let Ok(sig_bytes) = inner.0.signature().try_into() else {
                            continue;
                        };
                        let sig = Signature::from_bytes(sig_bytes);

                        if endpoint.verify(inner.0.payload_bytes(), &sig).is_err() {
                            continue;
                        }

                        found_valid = true;
                        break;
                    }
                }
            }

            if !found_valid {
                debug!("signature not from valid source");
                tx.send(None).await?;
                return Ok(());
            }

            // Generate and save session token.
            let mut token = SessionToken::default();
            rand::rng().fill_bytes(&mut token);

            if ctx
                .connections
                .insert_async(token, ConnectionState { did })
                .await
                .is_err()
            {
                debug!("already authenticated");
                tx.send(None).await?;
                return Ok(());
            }

            tx.send(Some(token)).await?;
        }
    }

    Ok(())
}
