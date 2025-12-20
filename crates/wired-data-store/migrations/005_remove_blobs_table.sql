-- Remove blobs table since FsStore now manages physical blob storage.
-- user_blobs and record_blobs remain for per-user ownership tracking.

-- First, remove foreign key references to blobs table.
-- SQLite doesn't support ALTER TABLE DROP FOREIGN KEY, so recreate the table.

-- Recreate user_blobs without the foreign key to blobs.
CREATE TABLE user_blobs_new (
    blob_id TEXT NOT NULL,
    owner_did TEXT NOT NULL,
    ref_count INTEGER NOT NULL DEFAULT 0,
    created INTEGER NOT NULL,
    PRIMARY KEY (blob_id, owner_did)
);

INSERT INTO user_blobs_new
SELECT
    blob_id,
    owner_did,
    ref_count,
    created
FROM user_blobs;

DROP TABLE user_blobs;
ALTER TABLE user_blobs_new RENAME TO user_blobs;

-- Drop the blobs table (FsStore handles physical blob storage).
DROP TABLE IF EXISTS blobs;
