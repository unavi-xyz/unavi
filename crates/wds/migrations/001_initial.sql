CREATE TABLE records (
    id TEXT PRIMARY KEY,
    creator TEXT NOT NULL,
    nonce BLOB NOT NULL,
    schema TEXT,
    created INTEGER NOT NULL,
    size INTEGER NOT NULL
);

CREATE INDEX idx_records_creator ON records (creator);
CREATE INDEX idx_records_schema ON records (schema);

CREATE TABLE pins (
    record_id TEXT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    owner TEXT NOT NULL,
    created INTEGER NOT NULL,
    expires INTEGER,
    PRIMARY KEY (owner, record_id)
);

CREATE TABLE user_quotas (
    owner TEXT PRIMARY KEY,
    bytes_used INTEGER NOT NULL DEFAULT 0,
    quota_bytes INTEGER NOT NULL
);
