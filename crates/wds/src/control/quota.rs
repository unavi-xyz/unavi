use std::sync::Arc;

use irpc::WithChannels;
use rusqlite::params;

use crate::{
    StoreContext,
    control::{
        ControlService,
        GetQuota,
        QuotaInfo,
        authenticate,
    },
    quota::ensure_quota_exists,
};

pub async fn get_quota(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<GetQuota, ControlService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let info = ctx
        .db
        .call(move |conn| {
            ensure_quota_exists(conn, &did_str)?;
            let (bytes_used, quota_bytes): (i64, i64) = conn.query_row(
                "SELECT bytes_used, quota_bytes FROM user_quotas WHERE owner = ?",
                params![&did_str],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            Ok(QuotaInfo {
                bytes_used,
                quota_bytes,
            })
        })
        .await?;

    tx.send(Ok(info)).await?;
    Ok(())
}
