use std::sync::Arc;

use irpc::WithChannels;

use crate::{
    StoreContext,
    control::{
        Announce,
        ControlService,
        RegistryId,
        authenticate,
    },
    error::ApiError,
    format::keys,
    signed_bytes::verify_did_signature,
};

/// Relays an announcer-signed [`Beacon`](crate::format::Beacon) into this
/// host's registry doc. The host is the sole writer; it checks the announcer is
/// the session holder and that the signature is valid, then stores the signed
/// payload verbatim so readers verify authenticity end-to-end.
pub async fn announce(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<Announce, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);

    let Some(registry_ns) = *ctx.registry.read() else {
        tx.send(Err(ApiError::Internal)).await?;
        return Ok(());
    };

    let Ok(beacon) = inner.beacon.payload() else {
        tx.send(Err(ApiError::Internal)).await?;
        return Ok(());
    };

    if beacon.did != did {
        tx.send(Err(ApiError::AccessDenied)).await?;
        return Ok(());
    }
    if !verify_did_signature(&inner.beacon, &beacon.did).await {
        tx.send(Err(ApiError::InvalidSignature)).await?;
        return Ok(());
    }

    let author = ctx.docs.api().author_default().await?;
    let Some(doc) = ctx.docs.api().open(registry_ns).await? else {
        tx.send(Err(ApiError::Internal)).await?;
        return Ok(());
    };

    let key = keys::beacon(beacon.space, &beacon.did);
    let value = postcard::to_stdvec(&inner.beacon)?;
    doc.set_bytes(author, key, value).await?;

    tx.send(Ok(())).await?;
    Ok(())
}

pub async fn registry_id(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<RegistryId, ControlService>,
) -> anyhow::Result<()> {
    let _did = authenticate!(ctx, inner, tx);
    let ns = *ctx.registry.read();
    tx.send(Ok(ns)).await?;
    Ok(())
}
