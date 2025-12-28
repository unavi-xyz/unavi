use std::sync::Arc;

use irpc::WithChannels;

use crate::{
    StoreContext,
    api::{ApiService, PinRecord, authenticate},
    quota::{QuotaExceeded, ensure_quota_exists, reserve_bytes},
};

pub async fn pin_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<PinRecord, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let db = ctx.db.pool();
    let record_id = inner.id.to_string();

    let mut db_tx = db.begin().await?;

    // Ensure user has a quota entry.
    ensure_quota_exists(&mut *db_tx, &did_str).await?;

    // Check if user already has this record pinned.
    let existing_pin = sqlx::query!(
        "SELECT record_id FROM record_pins WHERE owner = ? AND record_id = ?",
        did_str,
        record_id
    )
    .fetch_optional(&mut *db_tx)
    .await?;

    if existing_pin.is_some() {
        // Already pinned - just update expiration.
        sqlx::query!(
            "UPDATE record_pins SET expires = ? WHERE owner = ? AND record_id = ?",
            inner.expires,
            did_str,
            record_id
        )
        .execute(&mut *db_tx)
        .await?;
    } else {
        // New pin - reserve quota for existing envelopes.
        let total_size = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(size), 0) as total FROM envelopes WHERE record_id = ?",
            record_id
        )
        .fetch_one(&mut *db_tx)
        .await?;

        if total_size > 0
            && matches!(
                reserve_bytes(&mut *db_tx, &did_str, total_size).await,
                Err(QuotaExceeded)
            )
        {
            tx.send(Err("quota exceeded".into())).await?;
            return Ok(());
        }

        sqlx::query!(
            "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
            record_id,
            did_str,
            inner.expires
        )
        .execute(&mut *db_tx)
        .await?;
    }

    db_tx.commit().await?;

    tx.send(Ok(())).await?;
    Ok(())
}
