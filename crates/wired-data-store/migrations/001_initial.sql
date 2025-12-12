CREATE TABLE records (
    id TEXT PRIMARY KEY,
    creator TEXT NOT NULL,
    schema TEXT NOT NULL,
    created INTEGER NOT NULL,
    nonce BLOB NOT NULL,
    size INTEGER NOT NULL
);

CREATE INDEX idx_records_creator ON records(creator);
CREATE INDEX idx_records_schema ON records(schema);

CREATE TABLE blobs (
    id TEXT PRIMARY KEY,
    size INTEGER NOT NULL,
    created INTEGER NOT NULL
);

CREATE TABLE pins (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id TEXT NOT NULL REFERENCES records(id) ON DELETE CASCADE,
    created INTEGER NOT NULL,
    UNIQUE(record_id)
);

CREATE INDEX idx_pins_record ON pins(record_id);
