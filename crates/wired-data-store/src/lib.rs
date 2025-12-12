use std::path::PathBuf;

use anyhow::{Context, Result};

mod blob;
mod db;
mod pin;
mod record;

pub use blob::{Blob, BlobId};
pub use pin::{Pin, PinId};
pub use record::{Genesis, Record, RecordId};

use db::Database;

pub struct DataStore {
    db: Database,
    data_dir: PathBuf,
}

impl DataStore {
    /// Creates a new `DataStore` with the specified data directory.
    ///
    /// # Errors
    ///
    /// Returns error if directories cannot be created or database cannot be initialized.
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&data_dir).context("create data directory")?;
        std::fs::create_dir_all(data_dir.join("blobs")).context("create blobs directory")?;
        std::fs::create_dir_all(data_dir.join("records")).context("create records directory")?;

        let db_path = data_dir.join("index.db");
        let db = Database::new(&db_path).await?;

        Ok(Self { db, data_dir })
    }

    fn blobs_dir(&self) -> PathBuf {
        self.data_dir.join("blobs")
    }

    fn records_dir(&self) -> PathBuf {
        self.data_dir.join("records")
    }

    // Records.

    /// Creates a new record with the given genesis data.
    ///
    /// # Errors
    ///
    /// Returns error if record cannot be stored on filesystem or indexed in database.
    pub async fn create_record(&self, genesis: Genesis) -> Result<RecordId> {
        let record = Record::new(genesis);
        let snapshot = record.export_snapshot()?;

        // Store on filesystem.
        let path = self.record_path(&record.id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("create record shard directory")?;
        }
        std::fs::write(&path, &snapshot).context("write record file")?;

        // Index in database.
        let size = i64::try_from(snapshot.len()).context("snapshot size exceeds i64::MAX")?;
        sqlx::query(
            "INSERT INTO records (id, creator, schema, created, nonce, size) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(record.id.as_str())
        .bind(record.genesis.creator.as_str())
        .bind(record.genesis.schema.as_str())
        .bind(record.genesis.created.cast_signed())
        .bind(record.genesis.nonce.as_slice())
        .bind(size)
        .execute(self.db.pool())
        .await
        .context("insert record into database")?;

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

        // Query genesis data from database.
        let row: Option<(String, i64, Vec<u8>, String)> =
            sqlx::query_as("SELECT creator, created, nonce, schema FROM records WHERE id = ?")
                .bind(id.as_str())
                .fetch_optional(self.db.pool())
                .await
                .context("query record from database")?;

        let Some((creator, created, nonce, schema)) = row else {
            return Ok(None);
        };

        // Reconstruct genesis.
        let nonce: [u8; 16] = nonce
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid nonce length in database"))?;

        let genesis = Genesis {
            creator: creator.into(),
            created: created as u64,
            nonce,
            schema: schema.into(),
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
        let path = self.record_path(id);

        if path.exists() {
            std::fs::remove_file(&path).context("delete record file")?;
        }

        sqlx::query("DELETE FROM records WHERE id = ?")
            .bind(id.as_str())
            .execute(self.db.pool())
            .await
            .context("delete record from database")?;

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
        let id = BlobId::from_bytes(data);

        // Store on filesystem.
        let path = self.blob_path(&id);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("create blob shard directory")?;
        }
        std::fs::write(&path, data).context("write blob file")?;

        // Index in database.
        let size = i64::try_from(data.len()).context("blob size exceeds i64::MAX")?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        sqlx::query("INSERT OR IGNORE INTO blobs (id, size, created) VALUES (?, ?, ?)")
            .bind(id.as_str())
            .bind(size)
            .bind(now)
            .execute(self.db.pool())
            .await
            .context("insert blob into database")?;

        Ok(id)
    }

    /// Retrieves a blob by its ID.
    ///
    /// # Errors
    ///
    /// Returns error if blob cannot be read from filesystem.
    pub fn get_blob(&self, id: &BlobId) -> Result<Option<Vec<u8>>> {
        let path = self.blob_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&path).context("read blob file")?;
        Ok(Some(data))
    }

    fn blob_path(&self, id: &BlobId) -> PathBuf {
        let cid_str = id.as_str();
        let prefix = &cid_str[..2.min(cid_str.len())];
        self.blobs_dir().join(prefix).join(&cid_str)
    }

    // Pins.

    /// Pins a record by its ID.
    ///
    /// # Errors
    ///
    /// Returns error if pin cannot be created in database.
    ///
    /// # Panics
    ///
    /// Panics if system time is before UNIX epoch.
    pub async fn pin_record(&self, id: &RecordId) -> Result<PinId> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_secs()
            .cast_signed();

        let result = sqlx::query("INSERT INTO pins (record_id, created) VALUES (?, ?)")
            .bind(id.as_str())
            .bind(now)
            .execute(self.db.pool())
            .await
            .context("insert pin into database")?;

        Ok(PinId(result.last_insert_rowid() as u64))
    }

    /// Unpins a record by its pin ID.
    ///
    /// # Errors
    ///
    /// Returns error if pin cannot be deleted from database.
    pub async fn unpin_record(&self, pin_id: &PinId) -> Result<()> {
        sqlx::query("DELETE FROM pins WHERE id = ?")
            .bind(pin_id.0.cast_signed())
            .execute(self.db.pool())
            .await
            .context("delete pin from database")?;

        Ok(())
    }
}
