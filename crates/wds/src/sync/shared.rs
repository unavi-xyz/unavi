use anyhow::{Context, ensure};
use blake3::Hash;
use iroh_blobs::api::Store;
use loro::{LoroDoc, VersionVector};
use rusqlite::{Connection, params};
use xdid::{core::did::Did, resolver::DidResolver};

use wired_schemas::{Acl, Record, SCHEMA_ACL, SCHEMA_RECORD};

use crate::{
    auth::jwk::verify_jwk_signature,
    db::Database,
    quota,
    record::{
        envelope::Envelope,
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
    blobs: &Store,
    old_doc: &LoroDoc,
    new_doc: &LoroDoc,
    record: &Record,
    author: &Did,
    is_first_envelope: bool,
) -> anyhow::Result<()> {
    use std::collections::BTreeMap;

    use smol_str::SmolStr;

    use crate::record::schema::Schema;

    let old_frontiers = old_doc.state_frontiers();
    let new_frontiers = new_doc.state_frontiers();

    // Build schema map: container name -> schema.
    let mut schemas: BTreeMap<SmolStr, Schema> = BTreeMap::new();

    // Add built-in schemas.
    schemas.insert(
        "acl".into(),
        Schema::from_bytes(&SCHEMA_ACL.bytes)
            .map_err(|e| anyhow::anyhow!("failed to parse ACL schema: {e}"))?,
    );
    schemas.insert(
        "record".into(),
        Schema::from_bytes(&SCHEMA_RECORD.bytes)
            .map_err(|e| anyhow::anyhow!("failed to parse record schema: {e}"))?,
    );

    // Add record's schemas.
    for (container, schema_id) in &record.schemas {
        let schema = fetch_schema(blobs, schema_id)
            .await
            .map_err(|e| anyhow::anyhow!("failed to fetch schema: {e}"))?;
        schemas.insert(container.clone(), schema);
    }

    validate_diff(
        old_doc,
        new_doc,
        &old_frontiers,
        &new_frontiers,
        &schemas,
        author,
        is_first_envelope,
    )
    .map_err(|e| anyhow::anyhow!("schema validation failed: {e}"))?;

    Ok(())
}

/// Stores an envelope with quota tracking and schema validation.
pub async fn store_envelope(
    db: &Database,
    blobs: &Store,
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

    let size = i64::try_from(env_bytes.len()).context("envelope too large")?;
    let author_str = envelope.author().to_string();
    let from_vv = envelope.start_vv().encode();
    let to_vv = envelope.end_vv().encode();
    let ops = envelope.ops().to_vec();
    let payload_bytes = signed.payload_bytes().to_vec();
    let sig = signed.signature().to_vec();
    let new_vv = envelope.end_vv().encode();

    // Clone for use in closure.
    let record_id_owned = record_id.to_string();
    let record_clone = record.clone();
    let new_acl = Acl::load(&new_doc)?;

    db.call_mut(move |conn| {
        let tx = conn.transaction()?;

        let pinners = {
            let mut stmt = tx.prepare("SELECT owner FROM record_pins WHERE record_id = ?")?;
            let rows = stmt.query_map(params![&record_id_owned], |row| row.get::<_, String>(0))?;
            rows.filter_map(Result::ok).collect::<Vec<String>>()
        };

        if pinners.is_empty() {
            anyhow::bail!("record not pinned");
        }

        let mut any_reserved = false;
        for pinner in &pinners {
            if quota::reserve_bytes(&tx, pinner, size).is_ok() {
                any_reserved = true;
            }
        }
        if !any_reserved {
            anyhow::bail!("all pinners exceeded quota");
        }

        tx.execute(
            "INSERT INTO envelopes (record_id, author, from_vv, to_vv, ops, payload_bytes, signature, size)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![&record_id_owned, &author_str, &from_vv, &to_vv, &ops, &payload_bytes, &sig, size],
        )?;

        let updated = tx.execute(
            "UPDATE records SET vv = ? WHERE id = ?",
            params![&new_vv, &record_id_owned],
        )?;

        if updated == 0 {
            // Initialize a new record using metadata from the Loro doc.
            ensure!(
                record_clone.id()?.to_string() == record_id_owned,
                "record ID does not match"
            );

            let nonce: &[u8] = &record_clone.nonce;

            tx.execute(
                "INSERT INTO records (id, creator, nonce, timestamp, vv, size)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![&record_id_owned, &author_str, nonce, record_clone.timestamp, &new_vv, size],
            )?;

            // Index schemas (immutable after creation).
            for  schema_hash in record_clone.schemas.values() {
                let hash_str: String = schema_hash.to_string();

                tx.execute(
                    "INSERT OR IGNORE INTO record_schemas (record_id, schema_hash) VALUES (?, ?)",
                    params![&record_id_owned, &hash_str],
                )?;

                // Register as blob dependency for auto-pinning.
                tx.execute(
                    "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
                     VALUES (?, ?, 'schema')",
                    params![&record_id_owned, &hash_str],
                )?;
            }
        }

        // Update ACL read index from new doc state.
        update_acl_read_index(&tx, &record_id_owned, &new_acl)?;

        tx.commit()?;
        Ok(())
    })
    .await
}

/// Fetches all envelopes for a record.
pub(super) fn fetch_all_envelopes_sync(
    conn: &Connection,
    record_id: &str,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut stmt =
        conn.prepare("SELECT payload_bytes, signature FROM envelopes WHERE record_id = ?")?;

    let rows = stmt.query_map(params![record_id], |row| {
        Ok((row.get::<_, Vec<u8>>(0)?, row.get::<_, Vec<u8>>(1)?))
    })?;

    rows.map(|row| {
        let (payload_bytes, signature) = row?;
        let signed: SignedBytes<Envelope> = SignedBytes::from_parts(payload_bytes, signature);
        postcard::to_stdvec(&signed).map_err(Into::into)
    })
    .collect()
}

/// Fetches envelopes covering operations the remote is missing.
pub(super) fn fetch_envelopes_for_diff_sync(
    conn: &Connection,
    record_id: &str,
    local_vv: &VersionVector,
    remote_vv: &VersionVector,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut stmt =
        conn.prepare("SELECT payload_bytes, signature, to_vv FROM envelopes WHERE record_id = ?")?;

    let rows = stmt.query_map(params![record_id], |row| {
        Ok((
            row.get::<_, Vec<u8>>(0)?,
            row.get::<_, Vec<u8>>(1)?,
            row.get::<_, Vec<u8>>(2)?,
        ))
    })?;

    let mut result = Vec::new();
    for row in rows {
        let (payload_bytes, signature, to_vv_bytes) = row?;
        let env_to_vv = VersionVector::decode(&to_vv_bytes)?;

        // Include if remote doesn't have it and we do.
        if !remote_vv.includes_vv(&env_to_vv) && local_vv.includes_vv(&env_to_vv) {
            let signed: SignedBytes<Envelope> = SignedBytes::from_parts(payload_bytes, signature);
            result.push(postcard::to_stdvec(&signed)?);
        }
    }

    Ok(result)
}

/// Gets the version vector for a record.
pub(super) fn get_record_vv_sync(
    conn: &Connection,
    record_id: &str,
) -> anyhow::Result<VersionVector> {
    let result: Option<Vec<u8>> = conn
        .query_row(
            "SELECT vv FROM records WHERE id = ?",
            params![record_id],
            |row| row.get(0),
        )
        .ok();

    match result {
        Some(vv_bytes) => Ok(VersionVector::decode(&vv_bytes)?),
        None => Ok(VersionVector::new()),
    }
}

/// Gets blob dependency hashes for a record.
pub(super) fn get_blob_dep_hashes_sync(
    conn: &Connection,
    record_id: &str,
) -> anyhow::Result<Vec<Hash>> {
    let mut stmt = conn.prepare("SELECT blob_hash FROM record_blob_deps WHERE record_id = ?")?;

    let rows = stmt.query_map(params![record_id], |row| row.get::<_, String>(0))?;

    rows.map(|row| {
        let hash_str = row?;
        hash_str
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid blob hash: {hash_str}"))
    })
    .collect()
}

/// Async wrapper for fetching all envelopes.
pub(super) async fn fetch_all_envelopes(
    db: &Database,
    record_id: &str,
) -> anyhow::Result<Vec<Vec<u8>>> {
    let record_id = record_id.to_string();
    db.call(move |conn| fetch_all_envelopes_sync(conn, &record_id))
        .await
}

/// Async wrapper for getting record version vector.
pub(super) async fn get_record_vv(db: &Database, record_id: &str) -> anyhow::Result<VersionVector> {
    let record_id = record_id.to_string();
    db.call(move |conn| get_record_vv_sync(conn, &record_id))
        .await
}

/// Reconstructs a Loro document from stored envelopes (current state).
pub async fn reconstruct_current_doc(db: &Database, record_id: &str) -> anyhow::Result<LoroDoc> {
    let record_id = record_id.to_string();
    let existing = db
        .call(move |conn| fetch_all_envelopes_sync(conn, &record_id))
        .await?;

    let doc = LoroDoc::new();
    for env_bytes in existing {
        let signed: SignedBytes<Envelope> = postcard::from_bytes(&env_bytes)?;
        let env = signed.payload()?;
        doc.import(env.ops())?;
    }

    Ok(doc)
}

/// Updates the ACL read index for a record.
fn update_acl_read_index(conn: &Connection, record_id: &str, acl: &Acl) -> anyhow::Result<()> {
    // Update is_public flag.
    let is_public = i32::from(acl.is_public());
    conn.execute(
        "UPDATE records SET is_public = ? WHERE id = ?",
        params![is_public, record_id],
    )?;

    // Clear existing entries.
    conn.execute(
        "DELETE FROM record_acl_read WHERE record_id = ?",
        params![record_id],
    )?;

    // Insert all DIDs with read access (manage, write, and read).
    for did in acl
        .manage
        .iter()
        .chain(acl.write.iter())
        .chain(acl.read.iter())
    {
        let did_str = did.to_string();
        conn.execute(
            "INSERT OR IGNORE INTO record_acl_read (record_id, did) VALUES (?, ?)",
            params![record_id, &did_str],
        )?;
    }

    Ok(())
}
