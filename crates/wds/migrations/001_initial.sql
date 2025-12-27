CREATE TABLE records (
    id TEXT PRIMARY KEY,
    creator TEXT NOT NULL,
    nonce BLOB NOT NULL,
    timestamp INTEGER NOT NULL,
    vv BLOB NOT NULL,
    size INTEGER NOT NULL
);

CREATE INDEX idx_records_creator ON records (creator);

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
