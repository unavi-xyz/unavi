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

CREATE TABLE blobs (
    id TEXT PRIMARY KEY,
    size INTEGER NOT NULL,
    created INTEGER NOT NULL,
    owner_did TEXT NOT NULL
);

CREATE INDEX idx_blobs_owner ON blobs(owner_did);

CREATE TABLE pins (
    record_id TEXT NOT NULL REFERENCES records(id) ON DELETE CASCADE,
    created INTEGER NOT NULL,
    expires INTEGER,
    owner_did TEXT NOT NULL,
    PRIMARY KEY (owner_did, record_id)
);
