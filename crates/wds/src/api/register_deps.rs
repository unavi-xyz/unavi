use std::sync::Arc;

use irpc::WithChannels;

use crate::StoreContext;

use super::{ApiService, RegisterBlobDeps, authenticate};

pub async fn register_blob_deps(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<RegisterBlobDeps, ApiService>,
) -> anyhow::Result<()> {
    let _did = authenticate!(ctx, inner, tx);
    let db = ctx.db.pool();

    let record_id = inner.record_id.to_string();

    for (blob_hash, dep_type) in inner.deps {
        let hash_str: String = blob_hash.to_string();
        let dep_type_str: &str = dep_type.as_str();

        sqlx::query(
            "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
             VALUES (?, ?, ?)",
        )
        .bind(&record_id)
        .bind(&hash_str)
        .bind(dep_type_str)
        .execute(db)
        .await?;
    }

    tx.send(Ok(())).await?;

    Ok(())
}
