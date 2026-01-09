use std::sync::Arc;

use irpc::WithChannels;

use crate::{
    StoreContext,
    api::{ApiService, GetRecordPin, authenticate},
};

pub async fn get_record_pin(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<GetRecordPin, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let db = ctx.db.pool();
    let record_id = inner.id.to_string();

    match sqlx::query!(
        "SELECT expires FROM record_pins WHERE owner = ? AND record_id = ?",
        did_str,
        record_id
    )
    .fetch_optional(db)
    .await?
    {
        Some(found) => {
            tx.send(Ok(Some(found.expires))).await?;
        }
        None => {
            tx.send(Ok(None)).await?;
        }
    }

    Ok(())
}
