-- Signing keys delegated to WDS by users.
CREATE TABLE user_signing_keys (
    owner_did TEXT PRIMARY KEY,
    secret_key BLOB NOT NULL,
    algorithm TEXT NOT NULL,
    created INTEGER NOT NULL
);

-- Track sync state per peer per record.
CREATE TABLE sync_state (
    record_id TEXT NOT NULL,
    owner_did TEXT NOT NULL,
    endpoint_id BLOB NOT NULL,
    their_version BLOB,
    last_sync INTEGER NOT NULL,
    PRIMARY KEY (record_id, owner_did, endpoint_id),
    FOREIGN KEY (record_id, owner_did, endpoint_id)
    REFERENCES sync_peers (record_id, owner_did, endpoint_id) ON DELETE CASCADE
);

CREATE INDEX idx_sync_state_record ON sync_state (record_id, owner_did);
