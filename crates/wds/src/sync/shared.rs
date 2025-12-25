use anyhow::Context;
use loro::VersionVector;
use sqlx::{Pool, Sqlite};

use crate::{quota, record::envelope::Envelope, signed_bytes::SignedBytes};

/// Stores an envelope in the database with quota tracking.
/// Uses a transaction to ensure atomicity.
pub async fn store_envelope(
    db: &Pool<Sqlite>,
    record_id: &str,
    env_bytes: &[u8],
) -> anyhow::Result<()> {
    // Parse and validate envelope.
    let signed: SignedBytes<Envelope> = postcard::from_bytes(env_bytes)?;
    let envelope = signed.payload()?;
    // TODO: Verify signature.

    // TODO: Validate schema

    let mut tx = db.begin().await?;

    // Find who is pinning this record.
    let pinner = sqlx::query_scalar!(
        "SELECT owner FROM record_pins WHERE record_id = ? LIMIT 1",
        record_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(pinner) = pinner else {
        anyhow::bail!("record not pinned, cannot store envelope");
    };

    // Reserve quota.
    let size = i64::try_from(env_bytes.len()).context("envelope too large")?;
    quota::reserve_bytes(&mut *tx, &pinner, size)
        .await
        .map_err(|_| anyhow::anyhow!("quota exceeded"))?;

    // Store envelope.
    let author_str = envelope.author().to_string();
    let from_vv = envelope.start_vv().encode();
    let to_vv = envelope.end_vv().encode();
    let ops = envelope.ops();
    let payload_bytes = signed.payload_bytes();
    let sig = signed.signature();

    sqlx::query!(
        "INSERT INTO envelopes (record_id, author, from_vv, to_vv, ops, payload_bytes, signature, size)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        record_id,
        author_str,
        from_vv,
        to_vv,
        ops,
        payload_bytes,
        sig,
        size
    )
    .execute(&mut *tx)
    .await?;

    // Update record's vv.
    let new_vv = envelope.end_vv().encode();
    let rows = sqlx::query!("UPDATE records SET vv = ? WHERE id = ?", new_vv, record_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if rows == 0 {
        // Record doesn't exist yet - create from first envelope.
        // TODO: Extract nonce and timestamp from Loro doc.
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().cast_signed())
            .unwrap_or(0);
        let nonce: &[u8] = &[0u8; 16]; // TODO: Proper nonce handling.

        sqlx::query!(
            "INSERT INTO records (id, creator, nonce, timestamp, vv, size)
             VALUES (?, ?, ?, ?, ?, ?)",
            record_id,
            author_str,
            nonce,
            timestamp,
            new_vv,
            size
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Fetches all envelope bytes for a record.
pub async fn fetch_all_envelopes(
    db: &Pool<Sqlite>,
    record_id: &str,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let rows = sqlx::query!(
        "SELECT payload_bytes, signature FROM envelopes WHERE record_id = ?",
        record_id
    )
    .fetch_all(db)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let signed: SignedBytes<Envelope> =
            SignedBytes::from_parts(row.payload_bytes, row.signature);
        let bytes = postcard::to_stdvec(&signed)?;
        result.push(bytes);
    }

    Ok(result)
}

/// Fetches envelope bytes for envelopes that cover operations the remote is missing.
pub async fn fetch_envelopes_for_diff(
    db: &Pool<Sqlite>,
    record_id: &str,
    local_vv: &VersionVector,
    remote_vv: &VersionVector,
) -> anyhow::Result<Vec<Vec<u8>>> {
    // Get all envelopes for this record.
    let rows = sqlx::query!(
        "SELECT payload_bytes, signature, to_vv FROM envelopes WHERE record_id = ?",
        record_id
    )
    .fetch_all(db)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        // Decode the envelope's end version vector.
        let env_to_vv = VersionVector::decode(&row.to_vv)?;

        // Check if this envelope contains operations the remote doesn't have.
        // An envelope is needed if its to_vv has entries that remote_vv doesn't cover.
        if !remote_vv.includes_vv(&env_to_vv) {
            // Also check that we actually have these operations (sanity check).
            if local_vv.includes_vv(&env_to_vv) {
                let signed: SignedBytes<Envelope> =
                    SignedBytes::from_parts(row.payload_bytes, row.signature);
                let bytes = postcard::to_stdvec(&signed)?;
                result.push(bytes);
            }
        }
    }

    Ok(result)
}

/// Gets the version vector for a record.
pub async fn get_record_vv(db: &Pool<Sqlite>, record_id: &str) -> anyhow::Result<VersionVector> {
    let result = sqlx::query!("SELECT vv FROM records WHERE id = ?", record_id)
        .fetch_optional(db)
        .await?;

    match result {
        Some(row) => Ok(VersionVector::decode(&row.vv)?),
        None => Ok(VersionVector::new()),
    }
}
