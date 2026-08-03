CREATE TABLE user_quotas (
    owner TEXT PRIMARY KEY,
    bytes_used INTEGER NOT NULL DEFAULT 0,
    quota_bytes INTEGER NOT NULL
);

-- Explicit blob pins not covered by any hosted doc (uploads in progress,
-- build artifacts). Content is protected from GC while a pin is unexpired.
CREATE TABLE blob_pins (
    hash TEXT NOT NULL,
    owner TEXT NOT NULL,
    expires INTEGER NOT NULL,
    size INTEGER NOT NULL,
    PRIMARY KEY (owner, hash)
);

-- Docs this host replicates on behalf of an owner. Cost (metered on live
-- insert events) is the sum of the doc's entry content sizes, charged to the
-- owner's quota. Content of every entry in a hosted doc is protected from GC.
CREATE TABLE hosted_docs (
    ns TEXT NOT NULL,
    owner TEXT NOT NULL,
    bytes_used INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (owner, ns)
);

CREATE INDEX idx_hosted_docs_ns ON hosted_docs (ns);
