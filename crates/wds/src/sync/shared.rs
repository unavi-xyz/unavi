use std::collections::{BTreeMap, HashSet};

use anyhow::ensure;
use blake3::Hash;
use iroh_blobs::api::Store;
use loro::{LoroDoc, LoroValue, VersionVector};
use loro_schema::{Field, Schema};
use rusqlite::{Connection, params};
use smol_str::SmolStr;
use wired_schemas::{
    SCHEMA_ACL, SCHEMA_RECORD,
    surg::{Acl, Record},
};
use xdid::{core::did::Did, resolver::DidResolver};

use crate::{
    auth::jwk::verify_jwk_signature,
    db::Database,
    error::WdsError,
    quota,
    record::{
        envelope::Envelope,
        validate::{fetch_schema, validate_diff},
    },
    signed_bytes::SignedBytes,
};

/// Checks if the ACL was modified between old and new states.
fn acl_modified(old: &Acl, new: &Acl) -> bool {
    old.public != new.public
        || !slices_eq(old.managers(), new.managers())
        || !slices_eq(old.writers(), new.writers())
        || !slices_eq(old.readers(), new.readers())
}

fn slices_eq(a: &[wired_schemas::HydratedDid], b: &[wired_schemas::HydratedDid]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.0 == y.0)
}

/// Validates the signature of a signed envelope against the author's DID document.
async fn validate_signature(
    author: &Did,
    signature: &[u8],
    payload: &[u8],
) -> Result<(), WdsError> {
    let resolver = DidResolver::new().map_err(|e| WdsError::DidResolution(e.to_string()))?;
    let author_doc = resolver
        .resolve(author)
        .await
        .map_err(|e| WdsError::DidResolution(e.to_string()))?;

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

    Err(WdsError::InvalidSignature)
}

/// Validates document changes against all applicable schemas.
/// Returns the schema map for reuse in blob ref extraction.
async fn validate_schemas(
    blobs: &Store,
    old_doc: &LoroDoc,
    new_doc: &LoroDoc,
    record: &Record,
    author: &Did,
    is_first_envelope: bool,
) -> Result<BTreeMap<SmolStr, Schema>, WdsError> {
    let old_frontiers = old_doc.state_frontiers();
    let new_frontiers = new_doc.state_frontiers();

    // Build schema map: container name -> schema.
    let mut schemas: BTreeMap<SmolStr, Schema> = BTreeMap::new();

    // Add built-in schemas.
    schemas.insert(
        "acl".into(),
        Schema::from_bytes(&SCHEMA_ACL.bytes)
            .map_err(|e| WdsError::SchemaValidation(format!("failed to parse ACL schema: {e}")))?,
    );
    schemas.insert(
        "record".into(),
        Schema::from_bytes(&SCHEMA_RECORD.bytes).map_err(|e| {
            WdsError::SchemaValidation(format!("failed to parse record schema: {e}"))
        })?,
    );

    // Add record's schemas.
    for (container, schema_id) in &record.schemas {
        let schema = fetch_schema(blobs, schema_id)
            .await
            .map_err(|e| WdsError::SchemaValidation(format!("failed to fetch schema: {e}")))?;
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
    .map_err(|e| WdsError::SchemaValidation(e.to_string()))?;

    Ok(schemas)
}

/// Extracts all [`Field::BlobRef`] values from a Loro document by walking the schemas.
/// Returns hashes as hex strings (for database storage).
fn extract_blob_refs(doc: &LoroDoc, schemas: &BTreeMap<SmolStr, Schema>) -> HashSet<String> {
    let mut refs = HashSet::new();
    for (container_name, schema) in schemas {
        let value = doc.get_map(container_name.as_str()).get_deep_value();
        collect_blob_refs(&value, schema.layout(), &mut refs);
    }
    refs
}

fn collect_blob_refs(value: &LoroValue, field: &Field, refs: &mut HashSet<String>) {
    match field {
        Field::BlobRef => {
            if let LoroValue::Binary(bytes) = value {
                let slice: &[u8] = bytes.as_ref();
                if let Ok(arr) = <&[u8; 32]>::try_from(slice) {
                    refs.insert(Hash::from_bytes(*arr).to_string());
                }
            }
        }
        Field::Optional(inner) => {
            if !matches!(value, LoroValue::Null) {
                collect_blob_refs(value, inner, refs);
            }
        }
        Field::List(inner) | Field::MovableList(inner) => {
            if let LoroValue::List(items) = value {
                for item in items.iter() {
                    collect_blob_refs(item, inner, refs);
                }
            }
        }
        Field::Map(inner) => {
            if let LoroValue::Map(map) = value {
                for val in map.values() {
                    collect_blob_refs(val, inner, refs);
                }
            }
        }
        Field::Struct(fields) => {
            if let LoroValue::Map(map) = value {
                for (key, inner_field) in fields {
                    if let Some(val) = map.get(key.as_str()) {
                        collect_blob_refs(val, inner_field, refs);
                    }
                }
            }
        }
        Field::Tree(inner) => {
            if let LoroValue::List(nodes) = value {
                for node in nodes.iter() {
                    if let LoroValue::Map(m) = node
                        && let Some(meta) = m.get("meta")
                    {
                        collect_blob_refs(meta, inner, refs);
                    }
                }
            }
        }
        Field::Restricted { value: inner, .. } => collect_blob_refs(value, inner, refs),
        Field::Any | Field::Binary | Field::Bool | Field::F64 | Field::I64 | Field::String => {}
    }
}

/// Parameters for storing an envelope in the database.
struct StoreEnvelopeParams {
    record_id: String,
    author: String,
    from_vv: Vec<u8>,
    to_vv: Vec<u8>,
    ops: Vec<u8>,
    payload_bytes: Vec<u8>,
    signature: Vec<u8>,
    new_vv: Vec<u8>,
    size: i64,
    record: Record,
    acl: Acl,
    blob_refs: HashSet<String>,
}

/// Executes the database transaction to store an envelope.
fn store_envelope_tx(conn: &mut Connection, params: StoreEnvelopeParams) -> anyhow::Result<()> {
    let tx = conn.transaction()?;

    let pinners = {
        let mut stmt = tx.prepare("SELECT owner FROM record_pins WHERE record_id = ?")?;
        let rows = stmt.query_map(params![&params.record_id], |row| row.get::<_, String>(0))?;
        rows.filter_map(Result::ok).collect::<Vec<String>>()
    };

    if pinners.is_empty() {
        return Err(WdsError::NotPinned.into());
    }

    if !pinners
        .iter()
        .any(|p| quota::reserve_bytes(&tx, p, params.size).is_ok())
    {
        return Err(WdsError::QuotaExceeded.into());
    }

    tx.execute(
        "INSERT INTO envelopes (record_id, author, from_vv, to_vv, ops, payload_bytes, signature, size)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &params.record_id, &params.author, &params.from_vv, &params.to_vv,
            &params.ops, &params.payload_bytes, &params.signature, params.size
        ],
    )?;

    let updated = tx.execute(
        "UPDATE records SET vv = ? WHERE id = ?",
        params![&params.new_vv, &params.record_id],
    )?;

    if updated == 0 {
        initialize_new_record(&tx, &params)?;
    }

    update_acl_read_index(&tx, &params.record_id, &params.acl)?;
    update_blob_ref_deps(&tx, &params.record_id, &params.blob_refs)?;

    tx.commit()?;
    Ok(())
}

/// Initializes a new record in the database.
fn initialize_new_record(
    tx: &rusqlite::Transaction<'_>,
    params: &StoreEnvelopeParams,
) -> anyhow::Result<()> {
    ensure!(
        params.record.id()?.to_string() == params.record_id,
        "record ID does not match"
    );

    let nonce: &[u8] = &params.record.nonce;
    tx.execute(
        "INSERT INTO records (id, creator, nonce, timestamp, vv, size)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![
            &params.record_id,
            &params.author,
            nonce,
            params.record.timestamp,
            &params.new_vv,
            params.size
        ],
    )?;

    for schema_hash in params.record.schemas.values() {
        let hash_str = schema_hash.to_string();
        tx.execute(
            "INSERT OR IGNORE INTO record_schemas (record_id, schema_hash) VALUES (?, ?)",
            params![&params.record_id, &hash_str],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
             VALUES (?, ?, 'schema')",
            params![&params.record_id, &hash_str],
        )?;
    }
    Ok(())
}

/// Updates blob ref dependencies for a record.
fn update_blob_ref_deps(
    tx: &rusqlite::Transaction<'_>,
    record_id: &str,
    blob_refs: &HashSet<String>,
) -> anyhow::Result<()> {
    tx.execute(
        "DELETE FROM record_blob_deps WHERE record_id = ? AND dep_type = 'ref'",
        params![record_id],
    )?;
    for ref_hash in blob_refs {
        tx.execute(
            "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
             VALUES (?, ?, 'ref')",
            params![record_id, ref_hash],
        )?;
    }
    Ok(())
}

/// Stores an envelope with quota tracking and schema validation.
pub async fn store_envelope(
    db: &Database,
    blobs: &Store,
    record_id: &str,
    env_bytes: &[u8],
) -> Result<(), WdsError> {
    let signed: SignedBytes<Envelope> = postcard::from_bytes(env_bytes)?;
    let envelope = signed.payload().map_err(|e| WdsError::Other(e.into()))?;
    let author = envelope.author();

    validate_signature(author, signed.signature(), signed.payload_bytes()).await?;

    // Get current document state (BEFORE applying envelope).
    let old_doc = reconstruct_current_doc(db, record_id)
        .await
        .map_err(WdsError::Other)?;
    let is_first_envelope = old_doc.state_frontiers().is_empty();

    // Apply new envelope to get new state for diff computation.
    let new_doc = old_doc.fork();
    new_doc
        .import(envelope.ops())
        .map_err(|e| WdsError::Other(e.into()))?;

    // Check record-level write ACL against OLD state (prevent privilege escalation).
    if !is_first_envelope {
        let old_acl = Acl::load(&old_doc).map_err(WdsError::Other)?;
        if !old_acl.can_write(author) {
            return Err(WdsError::AccessDenied);
        }

        // Check if ACL was modified - requires manage permission.
        let new_acl = Acl::load(&new_doc).map_err(WdsError::Other)?;
        if acl_modified(&old_acl, &new_acl) && !old_acl.can_manage(author) {
            return Err(WdsError::AccessDenied);
        }
    }

    let record = Record::load(&new_doc).map_err(WdsError::Other)?;

    // First envelope author must match record creator.
    if is_first_envelope && record.creator.0 != *author {
        return Err(WdsError::AccessDenied);
    }

    let schemas = validate_schemas(
        blobs,
        &old_doc,
        &new_doc,
        &record,
        author,
        is_first_envelope,
    )
    .await?;
    let blob_refs = extract_blob_refs(&new_doc, &schemas);

    let params = StoreEnvelopeParams {
        record_id: record_id.to_string(),
        author: author.to_string(),
        from_vv: envelope.start_vv().encode(),
        to_vv: envelope.end_vv().encode(),
        ops: envelope.ops().to_vec(),
        payload_bytes: signed.payload_bytes().to_vec(),
        signature: signed.signature().to_vec(),
        new_vv: envelope.end_vv().encode(),
        size: i64::try_from(env_bytes.len())
            .map_err(|_| WdsError::Other(anyhow::anyhow!("envelope too large")))?,
        record,
        acl: Acl::load(&new_doc).map_err(WdsError::Other)?,
        blob_refs,
    };

    db.call_mut(move |conn| store_envelope_tx(conn, params))
        .await
        .map_err(WdsError::Other)?;

    Ok(())
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
    let is_public = i32::from(acl.public);
    conn.execute(
        "UPDATE records SET is_public = ? WHERE id = ?",
        params![is_public, record_id],
    )?;

    // Clear existing entries.
    conn.execute(
        "DELETE FROM record_acl_read WHERE record_id = ?",
        params![record_id],
    )?;

    // Insert all DIDs with read access (readers + writers + managers).
    for did in acl.readers() {
        let did_str = did.0.to_string();
        conn.execute(
            "INSERT OR IGNORE INTO record_acl_read (record_id, did) VALUES (?, ?)",
            params![record_id, &did_str],
        )?;
    }
    for did in acl.writers() {
        let did_str = did.0.to_string();
        conn.execute(
            "INSERT OR IGNORE INTO record_acl_read (record_id, did) VALUES (?, ?)",
            params![record_id, &did_str],
        )?;
    }
    for did in acl.managers() {
        let did_str = did.0.to_string();
        conn.execute(
            "INSERT OR IGNORE INTO record_acl_read (record_id, did) VALUES (?, ?)",
            params![record_id, &did_str],
        )?;
    }

    Ok(())
}
