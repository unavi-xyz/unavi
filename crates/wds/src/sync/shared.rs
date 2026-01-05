use anyhow::{Context, ensure};
use iroh_blobs::store::fs::FsStore;
use loro::{LoroDoc, VersionVector};
use sqlx::{Executor, Pool, Sqlite};
use xdid::{core::did::Did, resolver::DidResolver};

use crate::{
    auth::jwk::verify_jwk_signature,
    quota,
    record::{
        Record,
        acl::Acl,
        envelope::Envelope,
        schema::{SCHEMA_ACL, SCHEMA_RECORD},
        validate::{fetch_schema, validate_diff},
    },
    signed_bytes::SignedBytes,
};

/// Validates the signature of a signed envelope against the author's DID document.
async fn validate_signature(author: &Did, signature: &[u8], payload: &[u8]) -> anyhow::Result<()> {
    let resolver = DidResolver::new()?;
    let author_doc = resolver.resolve(author).await?;

    for method in author_doc.assertion_method.as_deref().unwrap_or_default() {
        let Some(map) = author_doc.resolve_verification_method(method) else {
            continue;
        };
        let Some(jwk) = &map.public_key_jwk else {
            continue;
        };
        if verify_jwk_signature(jwk, signature, payload) {
            return Ok(());
        }
    }

    anyhow::bail!("invalid envelope signature")
}

/// Validates document changes against all applicable schemas.
async fn validate_schemas(
    blobs: &FsStore,
    old_doc: &LoroDoc,
    new_doc: &LoroDoc,
    record: &Record,
    author: &Did,
    is_first_envelope: bool,
) -> anyhow::Result<()> {
    let old_frontiers = old_doc.state_frontiers();
    let new_frontiers = new_doc.state_frontiers();

    let mut schema_ids = vec![SCHEMA_ACL.hash, SCHEMA_RECORD.hash];
    schema_ids.extend(record.schemas.iter().copied());

    for schema_id in schema_ids {
        let schema = fetch_schema(blobs, &schema_id)
            .await
            .map_err(|e| anyhow::anyhow!("failed to fetch schema {schema_id}: {e}"))?;

        validate_diff(
            old_doc,
            new_doc,
            &old_frontiers,
            &new_frontiers,
            &schema,
            author,
            is_first_envelope,
        )
        .map_err(|e| anyhow::anyhow!("schema validation failed: {e}"))?;
    }

    Ok(())
}

/// Stores an envelope with quota tracking and schema validation.
pub async fn store_envelope(
    db: &Pool<Sqlite>,
    blobs: &FsStore,
    record_id: &str,
    env_bytes: &[u8],
) -> anyhow::Result<()> {
    let signed: SignedBytes<Envelope> = postcard::from_bytes(env_bytes)?;
    let envelope = signed.payload()?;
    let author = envelope.author();

    validate_signature(author, signed.signature(), signed.payload_bytes()).await?;

    // Get current document state (BEFORE applying envelope).
    let old_doc = reconstruct_current_doc(db, record_id).await?;
    let old_frontiers = old_doc.state_frontiers();
    let is_first_envelope = old_frontiers.is_empty();

    // Check record-level write ACL against OLD state (prevent privilege escalation).
    if !is_first_envelope {
        let acl = Acl::load(&old_doc)?;
        if !acl.can_write(author) {
            anyhow::bail!("access denied: write permission required");
        }
    }

    // Apply new envelope to get new state for diff computation.
    let new_doc = old_doc.fork();
    new_doc.import(envelope.ops())?;

    // Load record metadata from the new doc state.
    let record = Record::load(&new_doc)?;

    // Validate diff against schema restrictions.
    validate_schemas(
        blobs,
        &old_doc,
        &new_doc,
        &record,
        author,
        is_first_envelope,
    )
    .await?;

    let mut tx = db.begin().await?;

    let pinners = sqlx::query_scalar!(
        "SELECT owner FROM record_pins WHERE record_id = ?",
        record_id
    )
    .fetch_all(&mut *tx)
    .await?;

    if pinners.is_empty() {
        anyhow::bail!("record not pinned");
    }

    let size = i64::try_from(env_bytes.len()).context("envelope too large")?;
    let mut any_reserved = false;
    for pinner in &pinners {
        if quota::reserve_bytes(&mut *tx, pinner, size).await.is_ok() {
            any_reserved = true;
        }
    }
    if !any_reserved {
        anyhow::bail!("all pinners exceeded quota");
    }

    let author_str = envelope.author().to_string();
    let from_vv = envelope.start_vv().encode();
    let to_vv = envelope.end_vv().encode();
    let ops = envelope.ops();
    let payload_bytes = signed.payload_bytes();
    let sig = signed.signature();

    sqlx::query!(
        "INSERT INTO envelopes (record_id, author, from_vv, to_vv, ops, payload_bytes, signature, size)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        record_id, author_str, from_vv, to_vv, ops, payload_bytes, sig, size
    )
    .execute(&mut *tx)
    .await?;

    let new_vv = envelope.end_vv().encode();
    let updated = sqlx::query!("UPDATE records SET vv = ? WHERE id = ?", new_vv, record_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

    if updated == 0 {
        // Initialize a new record using metadata from the Loro doc.
        ensure!(
            record.id()?.to_string() == record_id,
            "record ID does not match"
        );

        let nonce: &[u8] = &record.nonce;

        sqlx::query!(
            "INSERT INTO records (id, creator, nonce, timestamp, vv, size)
             VALUES (?, ?, ?, ?, ?, ?)",
            record_id,
            author_str,
            nonce,
            record.timestamp,
            new_vv,
            size
        )
        .execute(&mut *tx)
        .await?;

        // Index schemas (immutable after creation).
        for schema_hash in &record.schemas {
            let hash_str = schema_hash.to_string();

            sqlx::query(
                "INSERT OR IGNORE INTO record_schemas (record_id, schema_hash) VALUES (?, ?)",
            )
            .bind(record_id)
            .bind(&hash_str)
            .execute(&mut *tx)
            .await?;

            // Register as blob dependency for auto-pinning.
            sqlx::query(
                "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
                 VALUES (?, ?, 'schema')",
            )
            .bind(record_id)
            .bind(&hash_str)
            .execute(&mut *tx)
            .await?;
        }
    }

    // Update ACL read index from new doc state.
    let new_acl = Acl::load(&new_doc)?;
    update_acl_read_index(&mut tx, record_id, &new_acl).await?;

    tx.commit().await?;
    Ok(())
}

/// Fetches all envelopes for a record.
pub(super) async fn fetch_all_envelopes<'e, E>(
    db: E,
    record_id: &str,
) -> anyhow::Result<Vec<Vec<u8>>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query!(
        "SELECT payload_bytes, signature FROM envelopes WHERE record_id = ?",
        record_id
    )
    .fetch_all(db)
    .await?;

    rows.into_iter()
        .map(|row| {
            let signed: SignedBytes<Envelope> =
                SignedBytes::from_parts(row.payload_bytes, row.signature);
            postcard::to_stdvec(&signed).map_err(Into::into)
        })
        .collect()
}

/// Fetches envelopes covering operations the remote is missing.
pub(super) async fn fetch_envelopes_for_diff<'e, E>(
    db: E,
    record_id: &str,
    local_vv: &VersionVector,
    remote_vv: &VersionVector,
) -> anyhow::Result<Vec<Vec<u8>>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let rows = sqlx::query!(
        "SELECT payload_bytes, signature, to_vv FROM envelopes WHERE record_id = ?",
        record_id
    )
    .fetch_all(db)
    .await?;

    let mut result = Vec::new();
    for row in rows {
        let env_to_vv = VersionVector::decode(&row.to_vv)?;

        // Include if remote doesn't have it and we do.
        if !remote_vv.includes_vv(&env_to_vv) && local_vv.includes_vv(&env_to_vv) {
            let signed: SignedBytes<Envelope> =
                SignedBytes::from_parts(row.payload_bytes, row.signature);
            result.push(postcard::to_stdvec(&signed)?);
        }
    }

    Ok(result)
}

/// Gets the version vector for a record.
pub(super) async fn get_record_vv<'e, E>(db: E, record_id: &str) -> anyhow::Result<VersionVector>
where
    E: Executor<'e, Database = Sqlite>,
{
    let result = sqlx::query!("SELECT vv FROM records WHERE id = ?", record_id)
        .fetch_optional(db)
        .await?;

    match result {
        Some(row) => Ok(VersionVector::decode(&row.vv)?),
        None => Ok(VersionVector::new()),
    }
}

/// Reconstructs a Loro document from stored envelopes (current state).
pub async fn reconstruct_current_doc(
    db: &Pool<Sqlite>,
    record_id: &str,
) -> anyhow::Result<LoroDoc> {
    let doc = LoroDoc::new();

    let existing = fetch_all_envelopes(db, record_id).await?;
    for env_bytes in existing {
        let signed: SignedBytes<Envelope> = postcard::from_bytes(&env_bytes)?;
        let env = signed.payload()?;
        doc.import(env.ops())?;
    }

    Ok(doc)
}

/// Updates the ACL read index for a record.
async fn update_acl_read_index(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    record_id: &str,
    acl: &Acl,
) -> anyhow::Result<()> {
    // Clear existing entries.
    sqlx::query("DELETE FROM record_acl_read WHERE record_id = ?")
        .bind(record_id)
        .execute(&mut **tx)
        .await?;

    // Insert all DIDs with read access (manage, write, and read).
    for did in acl
        .manage
        .iter()
        .chain(acl.write.iter())
        .chain(acl.read.iter())
    {
        let did_str = did.to_string();
        sqlx::query("INSERT OR IGNORE INTO record_acl_read (record_id, did) VALUES (?, ?)")
            .bind(record_id)
            .bind(&did_str)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}
