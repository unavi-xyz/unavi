use std::path::PathBuf;

use anyhow::{Context, Result};
use flume::Sender;
use iroh_blobs::store::fs::FsStore;
use smol_str::SmolStr;
use xdid::core::did::Did;

use crate::{
    BlobId, Genesis, MAX_BLOB_SIZE, Record, RecordId,
    db::Database,
    hash_did, quota,
    sync::{SyncEvent, SyncEventType, SyncPeer},
};

/// Per-user view over shared data store infrastructure.
///
/// Lightweight - cheap to create, can be cloned. Created via
/// `DataStore::view_for_user()`. Intended for per-user session
/// (reuse across requests).
#[derive(Clone)]
pub struct DataStoreView {
    pub(crate) blob_store: FsStore,
    pub(crate) db: Database,
    pub(crate) data_dir: PathBuf,
    pub(crate) owner_did: Did,
    pub(crate) sync_tx: Sender<SyncEvent>,
}

impl DataStoreView {
    /// Returns the DID that owns this data store view.
    #[must_use]
    pub const fn owner_did(&self) -> &Did {
        &self.owner_did
    }

    fn user_dir(&self) -> PathBuf {
        let did_hash = hash_did(&self.owner_did);
        self.data_dir.join("records").join(did_hash)
    }

    fn records_dir(&self) -> PathBuf {
        self.user_dir()
    }

    // Records.

    /// Creates a new record with the given genesis data.
    ///
    /// # Errors
    ///
    /// Returns error if record cannot be stored on filesystem or indexed in database,
    /// or if the user's storage quota would be exceeded.
    pub async fn create_record(&self, genesis: Genesis) -> Result<RecordId> {
        let record = Record::new(genesis);
        let snapshot = record.export_snapshot()?;
        let size = i64::try_from(snapshot.len()).context("snapshot size exceeds i64::MAX")?;
        let owner_did = self.owner_did.to_string();

        // Write file first (content-addressed, idempotent).
        let path = self.record_path(record.id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("create record shard directory")?;
        }
        std::fs::write(&path, &snapshot).context("write record file")?;

        // Transaction: ensure quota exists, reserve quota, insert record.
        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        quota::ensure_quota_exists(&mut *tx, &owner_did).await?;

        if let Err(e) = quota::reserve_bytes(&mut *tx, &owner_did, size).await {
            let _ = std::fs::remove_file(&path);
            return Err(anyhow::anyhow!(e));
        }

        let id = record.id.as_str();
        let creator = record.genesis.creator.to_string();
        let schema = record
            .genesis
            .schema
            .as_ref()
            .map(smol_str::SmolStr::as_str);
        let created = record.genesis.created.cast_signed();
        let nonce = record.genesis.nonce.as_slice();

        if let Err(e) = sqlx::query!(
            "INSERT INTO records (id, creator, schema, created, nonce, size, owner_did) VALUES (?, ?, ?, ?, ?, ?, ?)",
            id,
            creator,
            schema,
            created,
            nonce,
            size,
            owner_did
        )
        .execute(&mut *tx)
        .await
        {
            // Transaction will rollback on drop, just clean up the file.
            let _ = std::fs::remove_file(&path);
            return Err(e).context("insert record into database");
        }

        tx.commit().await.context("commit transaction")?;

        // Emit sync event.
        let _ = self.sync_tx.try_send(SyncEvent {
            record_id: record.id,
            owner_did: self.owner_did.clone(),
            event_type: SyncEventType::Created,
        });

        Ok(record.id)
    }

    /// Retrieves a record by its ID.
    ///
    /// # Errors
    ///
    /// Returns error if record cannot be read from filesystem or database.
    pub async fn get_record(&self, id: RecordId) -> Result<Option<Record>> {
        let path = self.record_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let snapshot = std::fs::read(&path).context("read record file")?;

        let record_id = id.as_str();
        let owner_did = self.owner_did.to_string();

        let row = sqlx::query!(
            "SELECT creator, created, nonce, schema FROM records WHERE id = ? AND owner_did = ?",
            record_id,
            owner_did
        )
        .fetch_optional(self.db.pool())
        .await
        .context("query record from database")?;

        let Some(row) = row else {
            return Ok(None);
        };

        let nonce: [u8; 16] = row
            .nonce
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid nonce length in database"))?;

        let creator = row
            .creator
            .parse()
            .map_err(|e| anyhow::anyhow!("invalid DID: {e}"))?;

        let created = u64::try_from(row.created)
            .map_err(|_| anyhow::anyhow!("invalid timestamp in database"))?;

        let genesis = Genesis {
            creator,
            created,
            nonce,
            schema: row.schema.map(SmolStr::from),
        };

        let mut record = Record::new(genesis);
        record.import_snapshot(&snapshot)?;

        Ok(Some(record))
    }

    /// Deletes a record by its ID.
    ///
    /// # Errors
    ///
    /// Returns error if record cannot be deleted from filesystem or database.
    pub async fn delete_record(&self, id: RecordId) -> Result<()> {
        let record_id = id.as_str();
        let owner_did = self.owner_did.to_string();
        let path = self.record_path(id);

        // Transaction: query, decrement refs, delete record, release quota.
        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Get record size before deletion.
        let size: Option<i64> = sqlx::query_scalar!(
            "SELECT size FROM records WHERE id = ? AND owner_did = ?",
            record_id,
            owner_did
        )
        .fetch_optional(&mut *tx)
        .await
        .context("query record size")?;

        // Get all blobs linked to this record.
        let linked_blobs: Vec<String> = sqlx::query_scalar!(
            "SELECT blob_id FROM record_blobs WHERE record_id = ? AND owner_did = ?",
            record_id,
            owner_did
        )
        .fetch_all(&mut *tx)
        .await
        .context("query linked blobs")?;

        // Decrement user_blob ref_counts.
        for blob_id in &linked_blobs {
            sqlx::query!(
                "UPDATE user_blobs SET ref_count = ref_count - 1 WHERE blob_id = ? AND owner_did = ?",
                blob_id,
                owner_did
            )
            .execute(&mut *tx)
            .await
            .context("decrement user_blob ref_count")?;
        }

        // Delete record (CASCADE will delete record_blobs entries).
        sqlx::query!(
            "DELETE FROM records WHERE id = ? AND owner_did = ?",
            record_id,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("delete record from database")?;

        // Release quota.
        if let Some(size) = size {
            quota::release_bytes(&mut *tx, &owner_did, size).await?;
        }

        tx.commit().await.context("commit transaction")?;

        // Delete file after successful commit.
        if path.exists() {
            std::fs::remove_file(&path).context("delete record file")?;
        }

        // Emit sync event.
        let _ = self.sync_tx.try_send(SyncEvent {
            record_id: id,
            owner_did: self.owner_did.clone(),
            event_type: SyncEventType::Deleted,
        });

        Ok(())
    }

    /// Apply Loro operations to an existing record.
    ///
    /// Internal/trusted method - caller must verify authorization.
    ///
    /// # Errors
    ///
    /// Returns error if record cannot be loaded, ops cannot be imported,
    /// or quota would be exceeded.
    pub(crate) async fn apply_ops(&self, record_id: RecordId, ops: &[u8]) -> Result<()> {
        let mut record = self
            .get_record(record_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("record not found"))?;

        // Get old size.
        let id = record_id.as_str();
        let owner_did = self.owner_did.to_string();

        let old_size: i64 = sqlx::query_scalar!(
            "SELECT size FROM records WHERE id = ? AND owner_did = ?",
            id,
            owner_did
        )
        .fetch_one(self.db.pool())
        .await
        .context("query record size")?;

        // Import ops into Loro doc.
        record.doc_mut().import(ops)?;

        let snapshot = record.export_snapshot()?;
        let new_size = i64::try_from(snapshot.len()).context("snapshot size exceeds i64::MAX")?;

        // Update size and quota atomically.
        let size_delta = new_size - old_size;

        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Adjust quota based on size delta.
        // If size_delta == 0, no quota adjustment needed.
        if size_delta > 0 {
            // Reserve additional bytes.
            quota::reserve_bytes(&mut *tx, &owner_did, size_delta).await?;
        } else if size_delta < 0 {
            // Release freed bytes.
            quota::release_bytes(&mut *tx, &owner_did, -size_delta).await?;
        }

        // Write new snapshot to file.
        let path = self.record_path(record_id);
        std::fs::write(&path, &snapshot).context("write updated record file")?;

        // Update record size.
        sqlx::query!(
            "UPDATE records SET size = ? WHERE id = ? AND owner_did = ?",
            new_size,
            id,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("update record size")?;

        tx.commit().await.context("commit transaction")?;

        Ok(())
    }

    fn record_path(&self, id: RecordId) -> PathBuf {
        let id_str = id.as_str();
        let prefix = &id_str[..2.min(id_str.len())];
        self.records_dir()
            .join(prefix)
            .join(format!("{id_str}.loro"))
    }

    // Blobs.

    /// Stores a blob and returns its content-addressed ID.
    ///
    /// Uses `FsStore` for physical storage (handles deduplication internally).
    /// Tracks per-user ownership in database for quota management.
    ///
    /// # Errors
    ///
    /// Returns error if blob cannot be stored or quota is exceeded.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn store_blob(&self, data: &[u8]) -> Result<BlobId> {
        if data.len() > MAX_BLOB_SIZE {
            return Err(anyhow::anyhow!(
                "blob size {} exceeds maximum {}",
                data.len(),
                MAX_BLOB_SIZE
            ));
        }

        let id = BlobId::from_bytes(data);
        let blob_id_str = id.as_str();
        let owner_did = self.owner_did.to_string();
        let size = i64::try_from(data.len()).context("blob size exceeds i64::MAX")?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        // Store in FsStore (handles deduplication internally).
        self.blob_store
            .add_slice(data)
            .await
            .context("store blob in FsStore")?;

        // Track ownership in database.
        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Check if user_blob exists.
        let user_blob_exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM user_blobs WHERE blob_id = ? AND owner_did = ?)",
            blob_id_str,
            owner_did
        )
        .fetch_one(&mut *tx)
        .await
        .context("check user_blob exists")?
            != 0;

        if !user_blob_exists {
            // Ensure quota record exists and reserve quota.
            quota::ensure_quota_exists(&mut *tx, &owner_did).await?;
            quota::reserve_bytes(&mut *tx, &owner_did, size).await?;

            // Create user_blob entry.
            sqlx::query!(
                "INSERT INTO user_blobs (blob_id, owner_did, ref_count, created) VALUES (?, ?, 0, ?)",
                blob_id_str,
                owner_did,
                now
            )
            .execute(&mut *tx)
            .await
            .context("insert user_blob into database")?;
        }

        tx.commit().await.context("commit transaction")?;

        Ok(id)
    }

    /// Retrieves a blob by its content-addressed ID.
    ///
    /// # Errors
    ///
    /// Returns error if blob cannot be read.
    pub async fn get_blob(&self, id: &BlobId) -> Result<Option<Vec<u8>>> {
        match self.blob_store.get_bytes(*id.hash()).await {
            Ok(bytes) => Ok(Some(bytes.to_vec())),
            Err(e) => {
                // Check if it's a "not found" error.
                let err_str = e.to_string();
                if err_str.contains("not found") || err_str.contains("NotFound") {
                    Ok(None)
                } else {
                    Err(e).context("read blob from FsStore")
                }
            }
        }
    }

    /// Links a blob to a record, incrementing the `user_blob` `ref_count`.
    ///
    /// # Errors
    ///
    /// Returns error if user hasn't uploaded this blob (no `user_blob` entry exists).
    pub async fn link_blob_to_record(&self, record_id: RecordId, blob_id: &BlobId) -> Result<()> {
        let record_id_str = record_id.as_str();
        let blob_id_str = blob_id.as_str();
        let owner_did = self.owner_did.to_string();

        // Transaction: check exists, insert link, increment ref_count.
        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Verify user_blob exists (user must have uploaded this blob).
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM user_blobs WHERE blob_id = ? AND owner_did = ?)",
            blob_id_str,
            owner_did
        )
        .fetch_one(&mut *tx)
        .await
        .context("check user_blob exists")?
            != 0;

        if !exists {
            return Err(anyhow::anyhow!(
                "blob not found for this user - must upload first"
            ));
        }

        // Insert record_blob link (ignore if already exists).
        sqlx::query!(
            "INSERT OR IGNORE INTO record_blobs (record_id, blob_id, owner_did) VALUES (?, ?, ?)",
            record_id_str,
            blob_id_str,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("insert record_blob link")?;

        // Increment user_blob ref_count.
        sqlx::query!(
            "UPDATE user_blobs SET ref_count = ref_count + 1 WHERE blob_id = ? AND owner_did = ?",
            blob_id_str,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("increment user_blob ref_count")?;

        tx.commit().await.context("commit transaction")?;

        Ok(())
    }

    // Pins.

    /// Pins a record by its ID with optional TTL.
    ///
    /// # Errors
    ///
    /// Returns error if pin cannot be created in database.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn pin_record(&self, id: RecordId, ttl: Option<u64>) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let expires = ttl.map(|secs| {
            let secs_i64 = i64::try_from(secs).unwrap_or(i64::MAX);
            now.saturating_add(secs_i64)
        });
        let record_id = id.as_str();
        let owner_did = self.owner_did.to_string();

        sqlx::query!(
            "INSERT OR REPLACE INTO pins (record_id, created, expires, owner_did) VALUES (?, ?, ?, ?)",
            record_id,
            now,
            expires,
            owner_did
        )
        .execute(self.db.pool())
        .await
        .context("insert pin into database")?;

        Ok(())
    }

    /// Unpins a record by its ID.
    ///
    /// # Errors
    ///
    /// Returns error if pin cannot be deleted from database.
    pub async fn unpin_record(&self, id: RecordId) -> Result<()> {
        let record_id = id.as_str();
        let owner_did = self.owner_did.to_string();

        sqlx::query!(
            "DELETE FROM pins WHERE record_id = ? AND owner_did = ?",
            record_id,
            owner_did
        )
        .execute(self.db.pool())
        .await
        .context("delete pin from database")?;

        Ok(())
    }

    /// Adds a sync peer to a pinned record.
    ///
    /// # Errors
    ///
    /// Returns error if database update fails.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn add_sync_peer(&self, record_id: RecordId, peer: &SyncPeer) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let record_id_str = record_id.as_str();
        let owner_did = self.owner_did.to_string();
        let endpoint_id_bytes = peer.as_bytes().as_slice();

        sqlx::query!(
            "INSERT OR IGNORE INTO sync_peers (record_id, owner_did, endpoint_id, created)
             VALUES (?, ?, ?, ?)",
            record_id_str,
            owner_did,
            endpoint_id_bytes,
            now
        )
        .execute(self.db.pool())
        .await
        .context("insert sync peer")?;

        Ok(())
    }

    /// Removes a sync peer from a record.
    ///
    /// # Errors
    ///
    /// Returns error if database delete fails.
    pub async fn remove_sync_peer(&self, record_id: RecordId, peer: &SyncPeer) -> Result<()> {
        let record_id_str = record_id.as_str();
        let owner_did = self.owner_did.to_string();
        let endpoint_id_bytes = peer.as_bytes().as_slice();

        sqlx::query!(
            "DELETE FROM sync_peers
             WHERE record_id = ? AND owner_did = ? AND endpoint_id = ?",
            record_id_str,
            owner_did,
            endpoint_id_bytes
        )
        .execute(self.db.pool())
        .await
        .context("delete sync peer")?;

        Ok(())
    }

    /// Lists all sync peers for a record.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails.
    pub async fn list_sync_peers(&self, record_id: RecordId) -> Result<Vec<SyncPeer>> {
        let record_id_str = record_id.as_str();
        let owner_did = self.owner_did.to_string();

        let rows: Vec<Vec<u8>> = sqlx::query_scalar!(
            "SELECT endpoint_id FROM sync_peers
             WHERE record_id = ? AND owner_did = ?
             ORDER BY created",
            record_id_str,
            owner_did
        )
        .fetch_all(self.db.pool())
        .await
        .context("query sync peers")?;

        rows.into_iter()
            .map(|bytes| {
                let arr: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| anyhow::anyhow!("endpoint_id must be exactly 32 bytes"))?;
                SyncPeer::from_bytes(&arr)
            })
            .collect()
    }

    /// Replaces entire sync peer list for a record.
    ///
    /// # Errors
    ///
    /// Returns error if database transaction fails.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn set_sync_peers(&self, record_id: RecordId, peers: &[SyncPeer]) -> Result<()> {
        let record_id_str = record_id.as_str();
        let owner_did = self.owner_did.to_string();

        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Clear existing peers.
        sqlx::query!(
            "DELETE FROM sync_peers WHERE record_id = ? AND owner_did = ?",
            record_id_str,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("delete existing sync peers")?;

        // Insert new peers.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        for peer in peers {
            let endpoint_id_bytes = peer.as_bytes().as_slice();
            sqlx::query!(
                "INSERT INTO sync_peers (record_id, owner_did, endpoint_id, created)
                 VALUES (?, ?, ?, ?)",
                record_id_str,
                owner_did,
                endpoint_id_bytes,
                now
            )
            .execute(&mut *tx)
            .await
            .context("insert sync peer")?;
        }

        tx.commit().await.context("commit transaction")?;

        Ok(())
    }

    // =========================================================================
    // Signing Key Management
    // =========================================================================

    /// Sets the signing key for this user.
    ///
    /// The signing key is used to sign updates on behalf of the user when
    /// syncing with peers.
    ///
    /// # Errors
    ///
    /// Returns error if database update fails.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn set_signing_key(&self, secret_key: &[u8], algorithm: &str) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let owner_did = self.owner_did.to_string();

        sqlx::query!(
            "INSERT INTO user_signing_keys (owner_did, secret_key, algorithm, created)
             VALUES (?, ?, ?, ?)
             ON CONFLICT (owner_did) DO UPDATE SET
                secret_key = excluded.secret_key,
                algorithm = excluded.algorithm,
                created = excluded.created",
            owner_did,
            secret_key,
            algorithm,
            now
        )
        .execute(self.db.pool())
        .await
        .context("upsert signing key")?;

        Ok(())
    }

    /// Gets the signing key for this user.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails.
    pub async fn get_signing_key(&self) -> Result<Option<(Vec<u8>, String)>> {
        let owner_did = self.owner_did.to_string();

        let row = sqlx::query!(
            "SELECT secret_key, algorithm FROM user_signing_keys WHERE owner_did = ?",
            owner_did
        )
        .fetch_optional(self.db.pool())
        .await
        .context("query signing key")?;

        Ok(row.map(|r| (r.secret_key, r.algorithm)))
    }

    /// Removes the signing key for this user.
    ///
    /// # Errors
    ///
    /// Returns error if database delete fails.
    pub async fn remove_signing_key(&self) -> Result<()> {
        let owner_did = self.owner_did.to_string();

        sqlx::query!(
            "DELETE FROM user_signing_keys WHERE owner_did = ?",
            owner_did
        )
        .execute(self.db.pool())
        .await
        .context("delete signing key")?;

        Ok(())
    }

    // =========================================================================
    // Envelope Storage
    // =========================================================================

    /// Stores an envelope and its op ranges to the database.
    ///
    /// # Arguments
    ///
    /// * `envelope` - The signed envelope to store
    /// * `op_ranges` - Vector of (`peer_id`, `counter_start`, `counter_end`) tuples
    ///
    /// # Errors
    ///
    /// Returns error if database insert fails.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub(crate) async fn store_envelope(
        &self,
        envelope: &crate::Envelope,
        op_ranges: &[(u64, u64, u64)],
    ) -> Result<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let record_id = envelope.record_id.as_str();
        let owner_did = self.owner_did.to_string();
        let author = envelope.author.to_string();
        let signature_alg = envelope.signature.alg.as_str();
        let ops_size = i64::try_from(envelope.ops.len()).context("ops size exceeds i64::MAX")?;

        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Insert envelope.
        let envelope_id: i64 = sqlx::query_scalar!(
            "INSERT INTO envelopes (record_id, owner_did, ops, from_version, to_version, author, signature_alg, signature_bytes, ops_size, created)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             RETURNING id",
            record_id,
            owner_did,
            envelope.ops,
            envelope.from_version,
            envelope.to_version,
            author,
            signature_alg,
            envelope.signature.bytes,
            ops_size,
            now
        )
        .fetch_one(&mut *tx)
        .await
        .context("insert envelope")?
        .ok_or_else(|| anyhow::anyhow!("insert did not return id"))?;

        // Insert op ranges.
        for &(peer_id, counter_start, counter_end) in op_ranges {
            let peer_id_i64 = i64::try_from(peer_id).context("peer_id exceeds i64::MAX")?;
            let start_i64 =
                i64::try_from(counter_start).context("counter_start exceeds i64::MAX")?;
            let end_i64 = i64::try_from(counter_end).context("counter_end exceeds i64::MAX")?;

            sqlx::query!(
                "INSERT INTO envelope_op_ranges (envelope_id, peer_id, counter_start, counter_end)
                 VALUES (?, ?, ?, ?)",
                envelope_id,
                peer_id_i64,
                start_i64,
                end_i64
            )
            .execute(&mut *tx)
            .await
            .context("insert envelope op range")?;
        }

        // Update snapshot counters.
        sqlx::query!(
            "UPDATE records SET ops_since_snapshot = ops_since_snapshot + 1,
                               bytes_since_snapshot = bytes_since_snapshot + ?
             WHERE id = ? AND owner_did = ?",
            ops_size,
            record_id,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("update snapshot counters")?;

        tx.commit().await.context("commit transaction")?;

        Ok(envelope_id)
    }

    /// Gets the current snapshot trigger counters for a record.
    ///
    /// # Returns
    ///
    /// Tuple of (`ops_since_snapshot`, `bytes_since_snapshot`, `latest_snapshot_num`)
    ///
    /// # Errors
    ///
    /// Returns error if database query fails or record not found.
    pub(crate) async fn get_snapshot_counters(
        &self,
        record_id: RecordId,
    ) -> Result<(i64, i64, i64)> {
        let id = record_id.as_str();
        let owner_did = self.owner_did.to_string();

        let row = sqlx::query!(
            "SELECT ops_since_snapshot, bytes_since_snapshot, latest_snapshot_num
             FROM records WHERE id = ? AND owner_did = ?",
            id,
            owner_did
        )
        .fetch_one(self.db.pool())
        .await
        .context("query snapshot counters")?;

        Ok((
            row.ops_since_snapshot,
            row.bytes_since_snapshot,
            row.latest_snapshot_num,
        ))
    }

    /// Stores a signed snapshot to the database.
    ///
    /// # Errors
    ///
    /// Returns error if database insert fails.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub(crate) async fn store_snapshot(&self, snapshot: &crate::SignedSnapshot) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let record_id = snapshot.record_id.as_str();
        let owner_did = self.owner_did.to_string();
        let snapshot_num =
            i64::try_from(snapshot.snapshot_num).context("snapshot_num exceeds i64::MAX")?;
        let signature_alg = snapshot.signature.alg.as_str();
        let snapshot_size = i64::try_from(snapshot.loro_snapshot.len())
            .context("snapshot size exceeds i64::MAX")?;

        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Insert snapshot.
        sqlx::query!(
            "INSERT INTO snapshots (record_id, owner_did, snapshot_num, version, genesis_bytes, snapshot_data, signature_alg, signature_bytes, snapshot_size, created)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            record_id,
            owner_did,
            snapshot_num,
            snapshot.version,
            snapshot.genesis_bytes,
            snapshot.loro_snapshot,
            signature_alg,
            snapshot.signature.bytes,
            snapshot_size,
            now
        )
        .execute(&mut *tx)
        .await
        .context("insert snapshot")?;

        // Reset counters and update latest_snapshot_num.
        sqlx::query!(
            "UPDATE records SET ops_since_snapshot = 0,
                               bytes_since_snapshot = 0,
                               latest_snapshot_num = ?
             WHERE id = ? AND owner_did = ?",
            snapshot_num,
            record_id,
            owner_did
        )
        .execute(&mut *tx)
        .await
        .context("reset snapshot counters")?;

        tx.commit().await.context("commit transaction")?;

        Ok(())
    }

    /// Gets the latest snapshot for a record.
    ///
    /// # Errors
    ///
    /// Returns error if database query fails.
    pub(crate) async fn get_latest_snapshot(
        &self,
        record_id: RecordId,
    ) -> Result<Option<crate::SignedSnapshot>> {
        let id = record_id.as_str();
        let owner_did = self.owner_did.to_string();

        let row = sqlx::query!(
            "SELECT snapshot_num, version, genesis_bytes, snapshot_data, signature_alg, signature_bytes
             FROM snapshots
             WHERE record_id = ? AND owner_did = ?
             ORDER BY snapshot_num DESC
             LIMIT 1",
            id,
            owner_did
        )
        .fetch_optional(self.db.pool())
        .await
        .context("query latest snapshot")?;

        let Some(row) = row else {
            return Ok(None);
        };

        let snapshot_num =
            u64::try_from(row.snapshot_num).context("snapshot_num exceeds u64::MAX")?;

        Ok(Some(crate::SignedSnapshot {
            record_id,
            snapshot_num,
            version: row.version,
            genesis_bytes: row.genesis_bytes,
            loro_snapshot: row.snapshot_data,
            signature: crate::Signature {
                alg: SmolStr::from(row.signature_alg),
                bytes: row.signature_bytes,
            },
        }))
    }

    /// Gets envelopes for a record since a given snapshot number.
    ///
    /// Returns envelopes in chronological order (oldest first).
    ///
    /// # Errors
    ///
    /// Returns error if database query fails.
    pub(crate) async fn get_envelopes_since_snapshot(
        &self,
        record_id: RecordId,
        since_snapshot_num: i64,
    ) -> Result<Vec<crate::Envelope>> {
        let id = record_id.as_str();
        let owner_did = self.owner_did.to_string();

        // Get the version of the snapshot we're starting from.
        let snapshot_version: Option<Vec<u8>> = if since_snapshot_num > 0 {
            sqlx::query_scalar!(
                "SELECT version FROM snapshots
                 WHERE record_id = ? AND owner_did = ? AND snapshot_num = ?",
                id,
                owner_did,
                since_snapshot_num
            )
            .fetch_optional(self.db.pool())
            .await
            .context("query snapshot version")?
        } else {
            None
        };

        // Get all envelopes created after the snapshot.
        // If no snapshot version, get all envelopes.
        let envelopes = if let Some(ref version) = snapshot_version {
            let rows = sqlx::query!(
                "SELECT ops, from_version, to_version, author, signature_alg, signature_bytes
                 FROM envelopes
                 WHERE record_id = ? AND owner_did = ? AND from_version >= ?
                 ORDER BY id ASC",
                id,
                owner_did,
                version
            )
            .fetch_all(self.db.pool())
            .await
            .context("query envelopes since snapshot")?;

            rows.into_iter()
                .map(|row| {
                    let author = row
                        .author
                        .parse()
                        .map_err(|e| anyhow::anyhow!("invalid author DID: {e}"))?;
                    Ok(crate::Envelope {
                        record_id,
                        ops: row.ops,
                        from_version: row.from_version,
                        to_version: row.to_version,
                        author,
                        signature: crate::Signature {
                            alg: SmolStr::from(row.signature_alg),
                            bytes: row.signature_bytes,
                        },
                    })
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            let rows = sqlx::query!(
                "SELECT ops, from_version, to_version, author, signature_alg, signature_bytes
                 FROM envelopes
                 WHERE record_id = ? AND owner_did = ?
                 ORDER BY id ASC",
                id,
                owner_did
            )
            .fetch_all(self.db.pool())
            .await
            .context("query all envelopes")?;

            rows.into_iter()
                .map(|row| {
                    let author = row
                        .author
                        .parse()
                        .map_err(|e| anyhow::anyhow!("invalid author DID: {e}"))?;
                    Ok(crate::Envelope {
                        record_id,
                        ops: row.ops,
                        from_version: row.from_version,
                        to_version: row.to_version,
                        author,
                        signature: crate::Signature {
                            alg: SmolStr::from(row.signature_alg),
                            bytes: row.signature_bytes,
                        },
                    })
                })
                .collect::<Result<Vec<_>>>()?
        };

        Ok(envelopes)
    }

    /// Deletes old envelopes and snapshots using lagged deletion strategy.
    ///
    /// Keeps snapshot N-1 and all envelopes from N-1 onwards.
    /// Deletes snapshot N-2 and earlier, and all their associated envelopes.
    ///
    /// # Errors
    ///
    /// Returns error if database operations fail.
    pub(crate) async fn gc_old_envelopes(&self, record_id: RecordId) -> Result<u64> {
        let id = record_id.as_str();
        let owner_did = self.owner_did.to_string();

        // Get latest snapshot number.
        let latest_num: i64 = sqlx::query_scalar!(
            "SELECT latest_snapshot_num FROM records WHERE id = ? AND owner_did = ?",
            id,
            owner_did
        )
        .fetch_one(self.db.pool())
        .await
        .context("query latest snapshot num")?;

        // Need at least 2 snapshots for lagged deletion.
        if latest_num < 2 {
            return Ok(0);
        }

        let keep_from_num = latest_num - 1;

        let mut tx = self.db.pool().begin().await.context("begin transaction")?;

        // Get the version of snapshot N-1.
        let keep_from_version: Vec<u8> = sqlx::query_scalar!(
            "SELECT version FROM snapshots
             WHERE record_id = ? AND owner_did = ? AND snapshot_num = ?",
            id,
            owner_did,
            keep_from_num
        )
        .fetch_one(&mut *tx)
        .await
        .context("query keep-from snapshot version")?;

        // Delete old snapshots (before N-1).
        sqlx::query!(
            "DELETE FROM snapshots
             WHERE record_id = ? AND owner_did = ? AND snapshot_num < ?",
            id,
            owner_did,
            keep_from_num
        )
        .execute(&mut *tx)
        .await
        .context("delete old snapshots")?;

        // Delete old envelopes (to_version < keep_from_version).
        // Using comparison on BLOB assumes lexicographic ordering works for version vectors.
        // This is a simplification; for production, decode and compare properly.
        let deleted = sqlx::query!(
            "DELETE FROM envelopes
             WHERE record_id = ? AND owner_did = ? AND to_version < ?",
            id,
            owner_did,
            keep_from_version
        )
        .execute(&mut *tx)
        .await
        .context("delete old envelopes")?;

        tx.commit().await.context("commit transaction")?;

        Ok(deleted.rows_affected())
    }
}
