use std::sync::Arc;

use irpc::WithChannels;
use time::OffsetDateTime;
use tracing::warn;
use wds::signed_bytes::verify_did_signature;

use crate::{
    RegistryContext,
    control::{
        RegistryService,
        Retract,
        Submit,
        authenticate,
    },
    error::RegistryError,
};

pub async fn submit(
    ctx: Arc<RegistryContext>,
    WithChannels { inner, tx, .. }: WithChannels<Submit, RegistryService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);

    if !ctx.config.permits(&did) {
        tx.send(Err(RegistryError::NotPermitted)).await?;
        return Ok(());
    }

    let Ok(submission) = inner.submission.payload() else {
        tx.send(Err(RegistryError::Malformed)).await?;
        return Ok(());
    };

    // The session holder must be the announcer; a registry does not let one
    // identity speak for another.
    if submission.did != did {
        tx.send(Err(RegistryError::NotPermitted)).await?;
        return Ok(());
    }

    if !verify_did_signature(&inner.submission, &submission.did).await {
        tx.send(Err(RegistryError::InvalidSignature)).await?;
        return Ok(());
    }

    let ceiling = (OffsetDateTime::now_utc() + ctx.config.max_retention).unix_timestamp();
    if submission.expires > ceiling {
        tx.send(Err(RegistryError::RetentionTooLong)).await?;
        return Ok(());
    }

    // Refreshing an existing entry is always allowed; only a new namespace
    // counts against the cap.
    let live = ctx.catalog.live(&ctx.docs, &ctx.blobs).await?;
    let held = live
        .iter()
        .filter(|s| s.did == did)
        .filter(|s| s.ns != submission.ns)
        .count();
    if held >= ctx.config.max_submissions_per_did {
        tx.send(Err(RegistryError::TooManySubmissions)).await?;
        return Ok(());
    }

    if let Err(err) = ctx
        .catalog
        .insert(&ctx.docs, &submission, &inner.submission)
        .await
    {
        warn!(?err, "failed writing submission");
        tx.send(Err(RegistryError::Internal)).await?;
        return Ok(());
    }

    ctx.request_rebuild();

    tx.send(Ok(())).await?;
    Ok(())
}

pub async fn retract(
    ctx: Arc<RegistryContext>,
    WithChannels { inner, tx, .. }: WithChannels<Retract, RegistryService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);

    let live = ctx.catalog.live(&ctx.docs, &ctx.blobs).await?;
    let owned = live.iter().any(|s| s.ns == inner.ns && s.did == did);

    if !owned {
        tx.send(Err(RegistryError::NotPermitted)).await?;
        return Ok(());
    }

    if let Err(err) = ctx.catalog.remove(&ctx.docs, inner.ns).await {
        warn!(?err, "failed removing submission");
        tx.send(Err(RegistryError::Internal)).await?;
        return Ok(());
    }

    ctx.request_rebuild();

    tx.send(Ok(())).await?;
    Ok(())
}
