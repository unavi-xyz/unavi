//! One handler per registry call.

use std::sync::Arc;

use iroh::EndpointId;
use irpc::WithChannels;
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{
    claim::MAX_CLAIM_BYTES,
    error::RegistryError,
    rpc::{
        Announce,
        Occupants,
        RegistryMessage,
        Retract,
        Submit,
    },
    server::Shared,
};

/// Runs one call from `remote`.
///
/// The caller's DID is read per call rather than once per connection, so a
/// handshake that finishes after the connection opened still counts.
pub async fn handle_message(
    shared: Arc<Shared>,
    remote: EndpointId,
    msg: RegistryMessage,
) -> anyhow::Result<()> {
    let caller = shared.bindings.did_of(remote);

    match msg {
        RegistryMessage::Submit(WithChannels { inner, tx, .. }) => {
            tx.send(submit(&shared, caller, inner).await).await?;
        }
        RegistryMessage::Retract(WithChannels { inner, tx, .. }) => {
            tx.send(retract(&shared, caller, inner).await).await?;
        }
        RegistryMessage::Announce(WithChannels { inner, tx, .. }) => {
            tx.send(announce(&shared, caller, remote, inner).await)
                .await?;
        }
        RegistryMessage::Occupants(WithChannels { inner, tx, .. }) => {
            tx.send(Ok(occupants(&shared, inner))).await?;
        }
        RegistryMessage::Views(WithChannels { tx, .. }) => {
            tx.send(Ok(shared.views.ids())).await?;
        }
    }

    Ok(())
}

fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

async fn submit(
    shared: &Shared,
    caller: Option<Did>,
    Submit {
        submission: signed,
        authorship,
    }: Submit,
) -> Result<(), RegistryError> {
    let did = caller.ok_or(RegistryError::Unauthenticated)?;

    if !shared.config.permits(&did) {
        return Err(RegistryError::NotPermitted);
    }
    if signed.payload_len() > MAX_CLAIM_BYTES {
        return Err(RegistryError::TooLarge);
    }

    let submission = signed.payload().map_err(|_| RegistryError::Malformed)?;
    submission.validate()?;

    // The connection holder must be the submitter; a registry does not let one
    // identity speak for another.
    if submission.did != did {
        return Err(RegistryError::NotPermitted);
    }

    let ceiling = (OffsetDateTime::now_utc() + shared.config.max_retention).unix_timestamp();
    if submission.expires > ceiling {
        return Err(RegistryError::RetentionTooLong);
    }
    if submission.expires <= now() {
        return Err(RegistryError::Expired);
    }

    // Offline checks first, so a request that fails them costs no fetch.
    let author = authorship
        .verify_namespace(submission.ns)
        .map_err(|_| RegistryError::NotAuthor)?;
    if author != did {
        return Err(RegistryError::NotAuthor);
    }

    signed
        .verify_did(&did, &shared.resolver)
        .await
        .map_err(|_| RegistryError::InvalidSignature)?;
    authorship
        .verify(submission.ns, &shared.resolver)
        .await
        .map_err(|_| RegistryError::NotAuthor)?;

    shared
        .catalog
        .insert(submission, signed, authorship, &shared.config)
        .await?;
    shared.request_rebuild();
    Ok(())
}

async fn retract(
    shared: &Shared,
    caller: Option<Did>,
    Retract { ns }: Retract,
) -> Result<(), RegistryError> {
    let did = caller.ok_or(RegistryError::Unauthenticated)?;
    shared.catalog.remove(ns, &did).await?;
    shared.request_rebuild();
    Ok(())
}

async fn announce(
    shared: &Shared,
    caller: Option<Did>,
    remote: EndpointId,
    Announce { presence: signed }: Announce,
) -> Result<(), RegistryError> {
    let did = caller.ok_or(RegistryError::Unauthenticated)?;

    if signed.payload_len() > MAX_CLAIM_BYTES {
        return Err(RegistryError::TooLarge);
    }
    let presence = signed.payload().map_err(|_| RegistryError::Malformed)?;

    // A DID may only say where it is, and only from the endpoint it names, or
    // it could send other peers to dial a victim.
    if presence.did != did || presence.endpoint != remote {
        return Err(RegistryError::NotPermitted);
    }

    let now = now();
    if presence.expires <= now {
        return Err(RegistryError::Expired);
    }
    let ttl = i64::try_from(shared.config.max_presence_ttl.as_secs()).unwrap_or(i64::MAX);
    if presence.expires > now.saturating_add(ttl) {
        return Err(RegistryError::RetentionTooLong);
    }

    signed
        .verify_did(&did, &shared.resolver)
        .await
        .map_err(|_| RegistryError::InvalidSignature)?;

    shared.presence.insert(&presence, signed, &shared.config)
}

fn occupants(
    shared: &Shared,
    Occupants { ns }: Occupants,
) -> Vec<unavi_identity::signed::Signed<crate::claim::Presence>> {
    shared.presence.occupants(ns)
}
