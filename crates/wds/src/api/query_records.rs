use std::{fmt::Write, sync::Arc};

use blake3::Hash;
use irpc::WithChannels;

use crate::StoreContext;

use super::{ApiService, QueryRecords, authenticate};

pub async fn query_records(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<QueryRecords, ApiService>,
) -> anyhow::Result<()> {
    let requester = authenticate!(ctx, inner, tx);
    let requester_str = requester.to_string();

    let db = ctx.db.pool();

    // Build dynamic query based on filters.
    // Use LEFT JOIN to include public records or records the requester has access to.
    let mut sql = String::from(
        "SELECT DISTINCT r.id FROM records r
         LEFT JOIN record_acl_read acl ON r.id = acl.record_id
         WHERE (r.is_public = 1 OR acl.did = ?)",
    );
    let mut binds: Vec<String> = vec![requester_str];

    if let Some(creator) = &inner.filter.creator {
        sql.push_str(" AND r.creator = ?");
        binds.push(creator.clone());
    }

    for (i, schema) in inner.filter.schemas.iter().enumerate() {
        write!(
            sql,
            " AND EXISTS (SELECT 1 FROM record_schemas s{i}
              WHERE s{i}.record_id = r.id AND s{i}.schema_hash = ?)"
        )
        .expect("write to string");
        binds.push(schema.to_string());
    }

    // Execute with dynamic bindings.
    let mut query = sqlx::query_scalar::<_, String>(&sql);
    for bind in &binds {
        query = query.bind(bind);
    }

    let ids: Vec<String> = query.fetch_all(db).await?;
    let hashes: Vec<Hash> = ids
        .into_iter()
        .filter_map(|s| Hash::from_hex(&s).ok())
        .collect();

    tx.send(Ok(hashes)).await?;
    Ok(())
}
