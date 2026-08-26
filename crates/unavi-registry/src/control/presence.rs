use std::sync::Arc;

use irpc::WithChannels;
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{
    RegistryContext,
    control::{
        Announce,
        Occupants,
        RegistryService,
        caller,
    },
    error::RegistryError,
};

pub async fn announce(
    ctx: Arc<RegistryContext>,
    caller: Option<Did>,
    WithChannels { inner, tx, .. }: WithChannels<Announce, RegistryService>,
) -> anyhow::Result<()> {
    let did = caller!(caller, tx);

    let Ok(presence) = inner.presence.payload() else {
        tx.send(Err(RegistryError::Malformed)).await?;
        return Ok(());
    };

    if presence.did != did {
        tx.send(Err(RegistryError::NotPermitted)).await?;
        return Ok(());
    }

    if presence.expires <= OffsetDateTime::now_utc().unix_timestamp() {
        tx.send(Err(RegistryError::Expired)).await?;
        return Ok(());
    }

    if inner
        .presence
        .verify(&presence.did, &ctx.resolver)
        .await
        .is_err()
    {
        tx.send(Err(RegistryError::InvalidSignature)).await?;
        return Ok(());
    }

    ctx.presence.insert(&presence, inner.presence).await;

    tx.send(Ok(())).await?;
    Ok(())
}

pub async fn occupants(
    ctx: Arc<RegistryContext>,
    WithChannels { inner, tx, .. }: WithChannels<Occupants, RegistryService>,
) -> anyhow::Result<()> {
    let occupants = ctx.presence.occupants(inner.ns).await;
    tx.send(Ok(occupants)).await?;
    Ok(())
}
