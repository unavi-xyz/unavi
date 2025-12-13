use std::path::PathBuf;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use xdid::core::did::Did;

mod blob;
mod db;
mod gc;
mod pin;
mod quota;
mod record;

pub use blob::{Blob, BlobId};
pub use gc::GarbageCollectStats;
pub use pin::Pin;
pub use quota::{QuotaExceeded, UserQuota, DEFAULT_QUOTA_BYTES};
pub use record::{Genesis, Record, RecordId};

use db::Database;

/// Maximum blob size: 512 MB.
pub const MAX_BLOB_SIZE: usize = 512 * 1024 * 1024;

pub struct DataStore {
    db: Database,
    data_dir: PathBuf,
    owner_did: Did,
}

pub(crate) fn hash_did(did: &Did) -> String {
    let hash = Sha256::digest(did.to_string().as_bytes());
    format!("{hash:x}")[..16].to_string()
}

impl DataStore {
    /// Creates a new `DataStore` with the specified data directory.
    ///
    /// # Errors
    ///
    /// Returns error if directories cannot be created or database cannot be initialized.
    pub async fn new(data_dir: PathBuf, owner_did: Did) -> Result<Self> {
        let did_hash = hash_did(&owner_did);
        let user_dir = data_dir.join("records").join(did_hash);

        std::fs::create_dir_all(&user_dir).context("create user records directory")?;
        std::fs::create_dir_all(data_dir.join("blobs")).context("create blobs directory")?;

        let db_path = data_dir.join("index.db");
        let db = Database::new(&db_path).await?;

        Ok(Self {
            db,
            data_dir,
            owner_did,
        })
    }

    /// Returns the DID that owns this data store.
    #[must_use]
    pub const fn owner_did(&self) -> &Did {
        &self.owner_did
    }

    fn user_dir(&self) -> PathBuf {
        let did_hash = hash_did(&self.owner_did);
        self.data_dir.join("records").join(did_hash)
    }

    fn blobs_dir(&self) -> PathBuf {
        self.data_dir.join("blobs")
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

        // Reserve quota before writing.
        quota::reserve_bytes(self.db.pool(), &owner_did, size)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        let path = self.record_path(&record.id);
        if let Some(parent) = path.parent()
            && let Err(e) = std::fs::create_dir_all(parent)
        {
            // Rollback quota on failure.
            let _ = quota::release_bytes(self.db.pool(), &owner_did, size).await;
            return Err(e).context("create record shard directory");
        }
        if let Err(e) = std::fs::write(&path, &snapshot) {
            // Rollback quota on failure.
            let _ = quota::release_bytes(self.db.pool(), &owner_did, size).await;
            return Err(e).context("write record file");
        }

        let id = record.id.as_str();
        let creator = record.genesis.creator.to_string();
        let schema = record.genesis.schema.as_str();
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
        .execute(self.db.pool())
        .await
        {
            // Rollback quota and file on failure.
            let _ = quota::release_bytes(self.db.pool(), &owner_did, size).await;
            let _ = std::fs::remove_file(&path);
            return Err(e).context("insert record into database");
        }

        Ok(record.id)
    }

    /// Retrieves a record by its ID.
    ///
    /// # Errors
    ///
    /// Returns error if record cannot be read from filesystem or database.
    pub async fn get_record(&self, id: &RecordId) -> Result<Option<Record>> {
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
            schema: row.schema.into(),
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
    pub async fn delete_record(&self, id: &RecordId) -> Result<()> {
        let record_id = id.as_str();
        let owner_did = self.owner_did.to_string();

        // Get record size before deletion.
        let size: Option<i64> =
            sqlx::query_scalar("SELECT size FROM records WHERE id = ? AND owner_did = ?")
                .bind(&record_id)
                .bind(&owner_did)
                .fetch_optional(self.db.pool())
                .await
                .context("query record size")?;

        // Get all blobs linked to this record.
        let linked_blobs: Vec<String> = sqlx::query_scalar(
            "SELECT blob_id FROM record_blobs WHERE record_id = ? AND owner_did = ?",
        )
        .bind(&record_id)
        .bind(&owner_did)
        .fetch_all(self.db.pool())
        .await
        .context("query linked blobs")?;

        // Decrement user_blob ref_counts.
        for blob_id in &linked_blobs {
            sqlx::query!(
                "UPDATE user_blobs SET ref_count = ref_count - 1 WHERE blob_id = ? AND owner_did = ?",
                blob_id,
                owner_did
            )
            .execute(self.db.pool())
            .await
            .context("decrement user_blob ref_count")?;
        }

        // Delete record file.
        let path = self.record_path(id);
        if path.exists() {
            std::fs::remove_file(&path).context("delete record file")?;
        }

        // Delete record (CASCADE will delete record_blobs entries).
        sqlx::query!(
            "DELETE FROM records WHERE id = ? AND owner_did = ?",
            record_id,
            owner_did
        )
        .execute(self.db.pool())
        .await
        .context("delete record from database")?;

        // Release quota after successful deletion.
        if let Some(size) = size {
            quota::release_bytes(self.db.pool(), &owner_did, size).await?;
        }

        Ok(())
    }

    fn record_path(&self, id: &RecordId) -> PathBuf {
        let cid_str = id.as_str();
        let prefix = &cid_str[..2.min(cid_str.len())];
        self.records_dir()
            .join(prefix)
            .join(format!("{cid_str}.loro"))
    }

    // Blobs.

    /// Stores a blob and returns its content-addressed ID.
    ///
    /// # Errors
    ///
    /// Returns error if blob cannot be stored on filesystem or indexed in database.
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

        // Check if physical blob exists.
        let blob_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM blobs WHERE id = ?)")
                .bind(&blob_id_str)
                .fetch_one(self.db.pool())
                .await
                .context("check blob exists")?;

        if blob_exists {
            // Verify CID matches (data integrity check).
            let stored_data = std::fs::read(self.blob_path(&id))
                .context("read existing blob for verification")?;
            let stored_id = BlobId::from_bytes(&stored_data);
            if stored_id != id {
                return Err(anyhow::anyhow!("blob CID mismatch"));
            }

            // Increment blob ref_count.
            sqlx::query!(
                "UPDATE blobs SET ref_count = ref_count + 1 WHERE id = ?",
                blob_id_str
            )
            .execute(self.db.pool())
            .await
            .context("increment blob ref_count")?;
        } else {
            // Write file to disk.
            let path = self.blob_path(&id);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).context("create blob shard directory")?;
            }
            std::fs::write(&path, data).context("write blob file")?;

            // Create blob entry with ref_count = 1.
            sqlx::query!(
                "INSERT INTO blobs (id, size, created, ref_count) VALUES (?, ?, ?, 1)",
                blob_id_str,
                size,
                now
            )
            .execute(self.db.pool())
            .await
            .context("insert blob into database")?;
        }

        // Check if user_blob exists.
        let user_blob_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_blobs WHERE blob_id = ? AND owner_did = ?)",
        )
        .bind(&blob_id_str)
        .bind(&owner_did)
        .fetch_one(self.db.pool())
        .await
        .context("check user_blob exists")?;

        if !user_blob_exists {
            // Reserve quota before creating user_blob (only charge when user claims the blob).
            quota::reserve_bytes(self.db.pool(), &owner_did, size)
                .await
                .map_err(|e| anyhow::anyhow!(e))?;

            // Create user_blob entry.
            if let Err(e) = sqlx::query!(
                "INSERT INTO user_blobs (blob_id, owner_did, ref_count, created) VALUES (?, ?, 0, ?)",
                blob_id_str,
                owner_did,
                now
            )
            .execute(self.db.pool())
            .await
            {
                // Rollback quota on failure.
                let _ = quota::release_bytes(self.db.pool(), &owner_did, size).await;
                return Err(e).context("insert user_blob into database");
            }
        }

        // Note: Don't increment user_blob ref_count here - that happens in link_blob_to_record.
        // TODO: This should be a transaction

        Ok(id)
    }

    /// Retrieves a blob by its content-addressed ID.
    ///
    /// # Errors
    ///
    /// Returns error if blob file cannot be read.
    pub fn get_blob(&self, id: &BlobId) -> Result<Option<Vec<u8>>> {
        let path = self.blob_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&path).context("read blob file")?;
        Ok(Some(data))
    }

    /// Links a blob to a record, incrementing the `user_blob` `ref_count`.
    ///
    /// # Errors
    ///
    /// Returns error if user hasn't uploaded this blob (no `user_blob` entry exists).
    pub async fn link_blob_to_record(&self, record_id: &RecordId, blob_id: &BlobId) -> Result<()> {
        let record_id_str = record_id.as_str();
        let blob_id_str = blob_id.as_str();
        let owner_did = self.owner_did.to_string();

        // Verify user_blob exists (user must have uploaded this blob).
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM user_blobs WHERE blob_id = ? AND owner_did = ?)",
        )
        .bind(&blob_id_str)
        .bind(&owner_did)
        .fetch_one(self.db.pool())
        .await
        .context("check user_blob exists")?;

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
        .execute(self.db.pool())
        .await
        .context("insert record_blob link")?;

        // Increment user_blob ref_count.
        sqlx::query!(
            "UPDATE user_blobs SET ref_count = ref_count + 1 WHERE blob_id = ? AND owner_did = ?",
            blob_id_str,
            owner_did
        )
        .execute(self.db.pool())
        .await
        .context("increment user_blob ref_count")?;

        Ok(())
    }

    fn blob_path(&self, id: &BlobId) -> PathBuf {
        let cid_str = id.as_str();
        let prefix = &cid_str[..2.min(cid_str.len())];
        self.blobs_dir().join(prefix).join(&cid_str)
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
    pub async fn pin_record(&self, id: &RecordId, ttl: Option<u64>) -> Result<()> {
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
    pub async fn unpin_record(&self, id: &RecordId) -> Result<()> {
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

    /// Runs garbage collection to remove expired pins, orphaned records, and orphaned blobs.
    ///
    /// # Errors
    ///
    /// Returns error if GC operations fail.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn garbage_collect(&self) -> Result<GarbageCollectStats> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let mut stats = GarbageCollectStats {
            pins_removed: 0,
            records_removed: 0,
            blobs_removed: 0,
            bytes_freed: 0,
        };

        // Step 1: Remove expired pins.
        let (expired_pins, pins_removed) = gc::remove_expired_pins(self.db.pool(), now).await?;
        stats.pins_removed = pins_removed;

        // Step 2: Remove unpinned records.
        for (record_id, owner_did) in expired_pins {
            if !gc::has_remaining_pins(self.db.pool(), &record_id, &owner_did).await?
                && let Some(size) = gc::remove_unpinned_record(
                    self.db.pool(),
                    &self.data_dir,
                    &record_id,
                    &owner_did,
                )
                .await?
            {
                stats.records_removed += 1;
                stats.bytes_freed += size;
            }
        }

        // Step 3: Remove orphaned user_blobs.
        gc::remove_orphaned_user_blobs(self.db.pool()).await?;

        // Step 4: Remove orphaned blobs.
        let (blobs_removed, bytes_freed) =
            gc::remove_orphaned_blobs(self.db.pool(), &self.blobs_dir()).await?;
        stats.blobs_removed = blobs_removed;
        stats.bytes_freed += bytes_freed;

        Ok(stats)
    }
}
