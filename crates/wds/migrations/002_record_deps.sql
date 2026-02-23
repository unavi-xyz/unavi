CREATE TABLE record_record_deps (
    record_id TEXT NOT NULL,
    dep_record_id TEXT NOT NULL,
    PRIMARY KEY (record_id, dep_record_id)
);

CREATE INDEX idx_record_record_deps_dep
ON record_record_deps (dep_record_id);
CREATE INDEX idx_record_record_deps_record
ON record_record_deps (record_id);
