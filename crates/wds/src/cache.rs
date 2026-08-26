//! Tags naming an expiry rather than an owner: the one dated GC root.
//!
//! Content read through the blob API has no owning root — it would be
//! collectible the moment the read returns. The deadline is fixed-width and
//! leads the name, so lexicographic tag order is chronological and the sweep
//! is a single range delete.

use std::time::Duration;

use blake3::Hash;
use iroh_blobs::api::Store as BlobStore;
use time::OffsetDateTime;

pub const DEFAULT_TTL: Duration = Duration::from_mins(10);

const PREFIX: &str = "cache/";

/// Deadlines round up to this granularity, so re-reads within a bucket rewrite
/// the same tag instead of finding and deleting its predecessor. Live tags per
/// hash are bounded by ttl divided by this.
const BUCKET: i64 = 600;

/// Wide enough for any `i64` second count, so every deadline pads to the same
/// length and the ordering holds.
const WIDTH: usize = 20;

fn name(hash: Hash, deadline: i64) -> String {
    format!("{PREFIX}{deadline:0WIDTH$}/{hash}")
}

fn deadline(ttl: Duration, now: i64) -> i64 {
    let expires = now.saturating_add(i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX));
    expires.saturating_add(BUCKET - 1) / BUCKET * BUCKET
}

/// Roots `hash` for at least `ttl`.
///
/// Called before a fetch, not after: `Downloader::download` takes no tag of its
/// own and the sweep lists partially written blobs, so a pass landing
/// mid-download would delete the content out from under the fetch.
pub async fn touch(blobs: &BlobStore, hash: Hash, ttl: Duration) -> anyhow::Result<()> {
    let deadline = deadline(ttl, OffsetDateTime::now_utc().unix_timestamp());
    blobs
        .tags()
        .set(name(hash, deadline), iroh_blobs::Hash::from(hash))
        .await?;
    Ok(())
}

/// Drops every cache tag whose deadline has passed. Content no other root
/// covers is reclaimed by the next blob GC pass.
pub async fn sweep(blobs: &BlobStore) -> anyhow::Result<u64> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let deleted = blobs
        .tags()
        .delete_range(PREFIX.to_string()..format!("{PREFIX}{now:0WIDTH$}"))
        .await?;
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: Hash = Hash::from_bytes([7; 32]);

    #[test]
    fn names_sort_chronologically() {
        let mut names = [
            name(HASH, 20_000_000_000),
            name(HASH, 1),
            name(HASH, 1_700_000_000),
        ];
        names.sort();
        assert_eq!(
            names,
            [
                name(HASH, 1),
                name(HASH, 1_700_000_000),
                name(HASH, 20_000_000_000)
            ],
            "a sweep by range depends on the deadline padding to a fixed width"
        );
    }

    #[test]
    fn a_passed_deadline_sorts_before_the_sweep_cutoff() {
        let now = 1_700_000_000;
        let cutoff = format!("{PREFIX}{now:0WIDTH$}");
        assert!(name(HASH, now - BUCKET) < cutoff);
        assert!(name(HASH, now + BUCKET) > cutoff);
    }

    #[test]
    fn a_reread_inside_a_bucket_rewrites_one_name() {
        let ttl = Duration::from_mins(10);
        let first = deadline(ttl, 1_700_000_000);
        assert_eq!(
            first,
            deadline(ttl, 1_700_000_000 + BUCKET - 1),
            "reads within a bucket must land on the same tag"
        );
        assert!(
            first >= 1_700_000_000 + ttl.as_secs().cast_signed(),
            "a bucket never expires a blob before its ttl"
        );
    }
}
