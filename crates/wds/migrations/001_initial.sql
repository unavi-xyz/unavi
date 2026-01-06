CREATE TABLE records (
    id TEXT PRIMARY KEY,
    creator TEXT NOT NULL,
    is_public INTEGER NOT NULL DEFAULT 0,
    nonce BLOB NOT NULL,
    size INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    vv BLOB NOT NULL
);

CREATE INDEX idx_records_creator ON records (creator);
CREATE INDEX idx_records_public ON records (is_public);

CREATE TABLE record_blob_deps (
    record_id TEXT NOT NULL,
    blob_hash TEXT NOT NULL,
    dep_type TEXT NOT NULL,
    PRIMARY KEY (record_id, blob_hash)
);

CREATE INDEX idx_record_blob_deps_blob ON record_blob_deps (blob_hash);
CREATE INDEX idx_record_blob_deps_record ON record_blob_deps (record_id);

-- Schema index (immutable after record creation).
CREATE TABLE record_schemas (
    record_id TEXT NOT NULL,
    schema_hash TEXT NOT NULL,
    PRIMARY KEY (record_id, schema_hash)
);

CREATE INDEX idx_record_schemas_hash ON record_schemas (schema_hash);

-- ACL read index (updated on ACL change).
CREATE TABLE record_acl_read (
    record_id TEXT NOT NULL,
    did TEXT NOT NULL,
    PRIMARY KEY (record_id, did)
);

CREATE INDEX idx_record_acl_read_did ON record_acl_read (did);

CREATE TABLE envelopes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id TEXT NOT NULL,
    author TEXT NOT NULL,
    from_vv BLOB NOT NULL,
    to_vv BLOB NOT NULL,
    ops BLOB NOT NULL,
    payload_bytes BLOB NOT NULL,
    signature BLOB NOT NULL,
    size INTEGER NOT NULL
);

CREATE TABLE blob_pins (
    hash TEXT NOT NULL,
    owner TEXT NOT NULL,
    expires INTEGER,
    size INTEGER NOT NULL,
    PRIMARY KEY (owner, hash)
);

CREATE TABLE record_pins (
    record_id TEXT NOT NULL,
    owner TEXT NOT NULL,
    expires INTEGER,
    PRIMARY KEY (owner, record_id)
);

CREATE TABLE user_quotas (
    owner TEXT PRIMARY KEY,
    bytes_used INTEGER NOT NULL DEFAULT 0,
    quota_bytes INTEGER NOT NULL
);
