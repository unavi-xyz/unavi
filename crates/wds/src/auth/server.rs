use std::{
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::bail;
use iroh::{
    EndpointId,
    Signature,
};
use irpc::{
    WithChannels,
    channel::oneshot,
};
use rand::RngCore;
use time::OffsetDateTime;
use tracing::debug;
use xdid::core::{
    did::Did,
    document::Document,
};

use crate::{
    ConnectionState,
    SESSION_TTL,
    SessionToken,
    StoreContext,
    WDS_SERVICE_TYPE,
    auth::{
        AnswerChallenge,
        AuthMessage,
        Challenge,
        HandlerState,
        Issued,
        Nonce,
        Pending,
        RequestChallenge,
        jwk::verify_jwk_signature,
    },
    resolve::resolve,
    signed_bytes::SignedBytes,
};

const NONCE_TTL: Duration = Duration::from_mins(3);

pub async fn handle_message(
    ctx: Arc<StoreContext>,
    state: Arc<HandlerState>,
    msg: AuthMessage,
) -> anyhow::Result<()> {
    match msg {
        AuthMessage::RequestChallenge(WithChannels { inner, tx, .. }) => {
            request_challenge(state, inner, tx).await
        }
        AuthMessage::AnswerChallenge(WithChannels { inner, tx, .. }) => {
            answer_challenge(ctx, state, inner, tx).await
        }
    }
}

async fn request_challenge(
    state: Arc<HandlerState>,
    RequestChallenge(did): RequestChallenge,
    tx: oneshot::Sender<Issued>,
) -> anyhow::Result<()> {
    let mut nonce = Nonce::default();
    rand::rng().fill_bytes(&mut nonce);

    let expires = (OffsetDateTime::now_utc() + NONCE_TTL).unix_timestamp();

    // Unanswered nonces need no reaper: the cache is capacity-bound, so it
    // evicts on its own, and `redeem_nonce` refuses anything past `expires`.
    if let Err((_, pending)) = state
        .nonces
        .put_async(nonce, Pending { did, expires })
        .await
    {
        bail!("Failed to generate nonce for {}", pending.did)
    }

    tx.send(Issued { nonce, expires }).await?;
    Ok(())
}

async fn answer_challenge(
    ctx: Arc<StoreContext>,
    state: Arc<HandlerState>,
    AnswerChallenge(signed): AnswerChallenge,
    tx: oneshot::Sender<Option<SessionToken>>,
) -> anyhow::Result<()> {
    let Some(did) = redeem_nonce(&state, &signed.payload()?, &ctx).await else {
        tx.send(None).await?;
        return Ok(());
    };

    let Some(doc) = resolve(&did).await else {
        tx.send(None).await?;
        return Ok(());
    };
    if !signature_is_authorized(&doc, &signed) {
        debug!("signature not from valid source");
        tx.send(None).await?;
        return Ok(());
    }

    let mut token = SessionToken::default();
    rand::rng().fill_bytes(&mut token);
    let expires = (OffsetDateTime::now_utc() + SESSION_TTL).unix_timestamp();

    if ctx
        .connections
        .insert_async(token, ConnectionState { did, expires })
        .await
        .is_err()
    {
        debug!("already authenticated");
        tx.send(None).await?;
        return Ok(());
    }

    tx.send(Some(token)).await?;
    Ok(())
}

/// Consumes the challenge's nonce and returns the DID it was issued to, once
/// the answer is shown to match that issuance and to be in-window.
async fn redeem_nonce(
    state: &HandlerState,
    challenge: &Challenge,
    ctx: &StoreContext,
) -> Option<Did> {
    if challenge.host != ctx.endpoint.id() {
        debug!("invalid host");
        return None;
    }

    // Taken, not read: a nonce answers exactly one challenge, so a captured
    // signature cannot be replayed for a second session.
    let Some((_, pending)) = state.nonces.remove_async(&challenge.nonce).await else {
        debug!("invalid nonce");
        return None;
    };

    if pending.did != challenge.did {
        debug!("wrong did");
        return None;
    }

    if challenge.expires != pending.expires {
        debug!("wrong expiry");
        return None;
    }

    if OffsetDateTime::now_utc().unix_timestamp() >= pending.expires {
        debug!("challenge expired");
        return None;
    }

    Some(pending.did)
}

fn signature_is_authorized(doc: &Document, signed: &SignedBytes<Challenge>) -> bool {
    signed_by_authentication_method(doc, signed) || signed_by_wds_service(doc, signed)
}

fn signed_by_authentication_method(doc: &Document, signed: &SignedBytes<Challenge>) -> bool {
    let Some(auth_methods) = &doc.authentication else {
        return false;
    };
    let signing_bytes = signed.signing_bytes();

    auth_methods.iter().any(|method| {
        doc.resolve_verification_method(method)
            .and_then(|map| map.public_key_jwk.as_ref())
            .is_some_and(|jwk| verify_jwk_signature(jwk, signed.signature(), &signing_bytes))
    })
}

/// Defined WDSes may authenticate on behalf of the DID, enabling cross-WDS
/// operations like reading or syncing. Written data still must be signed and
/// verified by an attestation method.
fn signed_by_wds_service(doc: &Document, signed: &SignedBytes<Challenge>) -> bool {
    let Some(services) = &doc.service else {
        return false;
    };
    let Ok(sig_bytes) = signed.signature().try_into() else {
        return false;
    };
    let sig = Signature::from_bytes(sig_bytes);
    let signing_bytes = signed.signing_bytes();

    services
        .iter()
        .filter(|service| service.typ.iter().any(|t| t == WDS_SERVICE_TYPE))
        .flat_map(|service| &service.service_endpoint)
        .filter_map(|id| EndpointId::from_str(id).ok())
        .any(|endpoint| endpoint.verify(&signing_bytes, &sig).is_ok())
}
