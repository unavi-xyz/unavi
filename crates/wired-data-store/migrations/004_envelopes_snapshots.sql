-- Persistent envelope storage for sync traceability.
CREATE TABLE envelopes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id TEXT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    owner_did TEXT NOT NULL,
    ops BLOB NOT NULL,
    from_version BLOB NOT NULL,
    to_version BLOB NOT NULL,
    author TEXT NOT NULL,
    signature_alg TEXT NOT NULL,
    signature_bytes BLOB NOT NULL,
    ops_size INTEGER NOT NULL,
    created INTEGER NOT NULL
);

CREATE INDEX idx_envelopes_record ON envelopes (record_id, owner_did);
CREATE INDEX idx_envelopes_created ON envelopes (created);

-- Op range mapping: which ops (by peer_id, counter range) are in each envelope.
CREATE TABLE envelope_op_ranges (
    envelope_id INTEGER NOT NULL REFERENCES envelopes (id) ON DELETE CASCADE,
    peer_id INTEGER NOT NULL,
    counter_start INTEGER NOT NULL,
    counter_end INTEGER NOT NULL,
    PRIMARY KEY (envelope_id, peer_id)
);

CREATE INDEX idx_op_ranges_peer ON envelope_op_ranges (peer_id, counter_start);

-- Signed snapshots for efficient sync base.
CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    record_id TEXT NOT NULL REFERENCES records (id) ON DELETE CASCADE,
    owner_did TEXT NOT NULL,
    snapshot_num INTEGER NOT NULL,
    version BLOB NOT NULL,
    genesis_bytes BLOB NOT NULL,
    snapshot_data BLOB NOT NULL,
    signature_alg TEXT NOT NULL,
    signature_bytes BLOB NOT NULL,
    snapshot_size INTEGER NOT NULL,
    created INTEGER NOT NULL,
    UNIQUE (record_id, owner_did, snapshot_num)
);

CREATE INDEX idx_snapshots_record ON snapshots (record_id, owner_did);

-- Tracking columns for snapshot triggers.
ALTER TABLE records ADD COLUMN ops_since_snapshot INTEGER NOT NULL DEFAULT 0;
ALTER TABLE records ADD COLUMN bytes_since_snapshot INTEGER NOT NULL DEFAULT 0;
ALTER TABLE records ADD COLUMN latest_snapshot_num INTEGER NOT NULL DEFAULT 0;
