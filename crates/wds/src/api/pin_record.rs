use std::sync::Arc;

use irpc::WithChannels;

use crate::{
    StoreContext,
    api::{ApiService, PinRecord, authenticate},
};

pub async fn pin_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<PinRecord, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let db = ctx.db.pool();
    let record_id = inner.id.to_string();

    // Insert or update the record pin.
    sqlx::query!(
        "INSERT INTO record_pins (record_id, owner, expires)
         VALUES (?, ?, ?)
         ON CONFLICT (owner, record_id) DO UPDATE SET expires = excluded.expires",
        record_id,
        did_str,
        inner.expires
    )
    .execute(db)
    .await?;

    tx.send(Ok(())).await?;
    Ok(())
}
