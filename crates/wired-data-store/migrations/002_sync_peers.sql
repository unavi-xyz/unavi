-- Iroh endpoint identity (Ed25519 32-byte secret key).
CREATE TABLE iroh_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    secret_key BLOB NOT NULL,
    created INTEGER NOT NULL
);

-- Sync peers associated with pinned records.
-- Automatically cleaned up when pin is deleted via CASCADE.
CREATE TABLE sync_peers (
    record_id TEXT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    owner_did TEXT NOT NULL,
    endpoint_id BLOB NOT NULL,
    created INTEGER NOT NULL,
    PRIMARY KEY (record_id, owner_did, endpoint_id),
    FOREIGN KEY (record_id, owner_did) REFERENCES pins (
        record_id, owner_did
    ) ON DELETE CASCADE
);

CREATE INDEX idx_sync_peers_record ON sync_peers (record_id, owner_did);
CREATE INDEX idx_sync_peers_endpoint ON sync_peers (endpoint_id);
