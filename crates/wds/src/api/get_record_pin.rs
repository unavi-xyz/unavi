use std::sync::Arc;

use irpc::WithChannels;
use rusqlite::params;

use crate::{
    StoreContext,
    api::{ApiError, ApiService, GetRecordPin, authenticate},
};

pub async fn get_record_pin(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<GetRecordPin, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let record_id = inner.id.to_string();

    let expires = ctx
        .db
        .call(move |conn| {
            let result: Option<i64> = conn
                .query_row(
                    "SELECT expires FROM record_pins WHERE owner = ? AND record_id = ?",
                    params![&did_str, &record_id],
                    |row| row.get(0),
                )
                .ok();

            Ok(result)
        })
        .await?;

    tx.send(Ok(expires)).await?;
    Ok(())
}
