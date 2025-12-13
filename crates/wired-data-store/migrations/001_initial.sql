CREATE TABLE records (
    id TEXT PRIMARY KEY,
    creator TEXT NOT NULL,
    schema TEXT NOT NULL,
    created INTEGER NOT NULL,
    nonce BLOB NOT NULL,
    size INTEGER NOT NULL,
    owner_did TEXT NOT NULL
);

CREATE INDEX idx_records_creator ON records(creator);
CREATE INDEX idx_records_schema ON records(schema);
CREATE INDEX idx_records_owner ON records(owner_did);

-- Physical blobs on disk (content-addressed, shared across users).
CREATE TABLE blobs (
    id TEXT PRIMARY KEY,
    size INTEGER NOT NULL,
    created INTEGER NOT NULL,
    ref_count INTEGER NOT NULL DEFAULT 0
);

-- User's claim to a blob (proof they uploaded it).
CREATE TABLE user_blobs (
    blob_id TEXT NOT NULL REFERENCES blobs(id),
    owner_did TEXT NOT NULL,
    ref_count INTEGER NOT NULL DEFAULT 0,
    created INTEGER NOT NULL,
    PRIMARY KEY (blob_id, owner_did)
);

CREATE INDEX idx_user_blobs_owner ON user_blobs(owner_did);

-- Which records reference which blobs (for a specific user).
CREATE TABLE record_blobs (
    record_id TEXT NOT NULL REFERENCES records(id) ON DELETE CASCADE,
    blob_id TEXT NOT NULL,
    owner_did TEXT NOT NULL,
    PRIMARY KEY (record_id, blob_id, owner_did),
    FOREIGN KEY (blob_id, owner_did) REFERENCES user_blobs(blob_id, owner_did)
);

CREATE INDEX idx_record_blobs_blob ON record_blobs(blob_id, owner_did);

CREATE TABLE pins (
    record_id TEXT NOT NULL REFERENCES records(id) ON DELETE CASCADE,
    created INTEGER NOT NULL,
    expires INTEGER,
    owner_did TEXT NOT NULL,
    PRIMARY KEY (owner_did, record_id)
);

-- Per-user storage tracking and quotas.
CREATE TABLE user_quotas (
    owner_did TEXT PRIMARY KEY,
    bytes_used INTEGER NOT NULL DEFAULT 0,
    quota_bytes INTEGER NOT NULL,
    created INTEGER NOT NULL,
    updated INTEGER NOT NULL
);
