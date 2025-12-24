use std::sync::Arc;

use blake3::Hash;
use irpc::WithChannels;
use loro::{ExportMode, LoroDoc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{
    StoreContext,
    api::{ApiService, CreateRecord, authenticate},
    quota::{QuotaExceeded, reserve_bytes},
};

type RecordNonce = [u8; 16];

#[derive(Debug, Serialize, Deserialize)]
struct RecordMeta {
    creator: Did,
    nonce: RecordNonce,
    schema: Option<String>,
    timestamp: i64,
}

impl RecordMeta {
    const SIZE: usize = size_of::<Self>();

    fn new(creator: Did, schema: Option<String>) -> Self {
        let mut nonce = RecordNonce::default();
        rand::rng().fill(&mut nonce);

        Self {
            creator,
            nonce,
            schema,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    fn id(&self) -> anyhow::Result<Hash> {
        let bytes = postcard::to_vec::<_, { Self::SIZE }>(self)?;
        Ok(blake3::hash(&bytes))
    }
}

pub async fn create_record(
    ctx: Arc<StoreContext>,
    WithChannels { inner, tx, .. }: WithChannels<CreateRecord, ApiService>,
) -> anyhow::Result<()> {
    let did = authenticate!(ctx, inner, tx);
    let did_str = did.to_string();

    let doc = LoroDoc::new();
    let data = doc.export(ExportMode::all_updates())?;
    let size = i64::try_from(data.len())?;

    let meta = RecordMeta::new(did, None);
    let record_id = meta.id()?;

    let db = ctx.db.pool();
    let mut db_tx = db.begin().await?;

    let record_id_hex = record_id.to_hex();
    let record_id_str = record_id_hex.as_str();
    let nonce = meta.nonce.as_slice();

    sqlx::query!(
        "INSERT INTO records (id, creator, nonce, schema, timestamp, size) VALUES (?, ?, ?, ?, ?, ?)",
        record_id_str,
        did_str,
        nonce,
        meta.schema,
        meta.timestamp,
        size
    )
    .execute(&mut *db_tx)
    .await?;

    if matches!(
        reserve_bytes(&mut *db_tx, &did_str, size).await,
        Err(QuotaExceeded)
    ) {
        tx.send(Err("quota exceeded".into())).await?;
        return Ok(());
    }

    db_tx.commit().await?;

    tx.send(Ok(record_id)).await?;

    Ok(())
}
