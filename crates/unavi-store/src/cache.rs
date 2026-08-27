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

/// Deadlines round up to this granularity, so re-reads share a tag instead of
/// leaving one per read. A hash holds at most `ttl / BUCKET + 1` live tags.
const BUCKET: i64 = 600;

/// Wide enough for any `i64` second count, so every deadline pads to the same
/// length and the ordering holds.
const WIDTH: usize = 20;

/// Roots blobs against garbage collection for a bounded time.
#[derive(Clone, Debug)]
pub struct Cache(BlobStore);

impl Cache {
    #[must_use]
    pub const fn new(blobs: BlobStore) -> Self {
        Self(blobs)
    }

    /// Roots `hash` for at least `ttl`.
    ///
    /// Called before a fetch, not after: `Downloader::download` takes no tag of
    /// its own and the sweep lists partially written blobs, so a pass landing
    /// mid-download would delete the content out from under the fetch.
    pub async fn touch(&self, hash: Hash, ttl: Duration) -> anyhow::Result<()> {
        let deadline = deadline(ttl, now());
        self.0
            .tags()
            .set(name(hash, deadline), iroh_blobs::Hash::from(hash))
            .await?;
        Ok(())
    }

    /// Drops every cache tag whose deadline has passed. Content no other root
    /// covers is reclaimed by the next blob GC pass.
    pub async fn sweep(&self) -> anyhow::Result<u64> {
        let now = now();
        let deleted = self
            .0
            .tags()
            .delete_range(PREFIX.to_string()..format!("{PREFIX}{now:0WIDTH$}"))
            .await?;
        Ok(deleted)
    }
}

fn name(hash: Hash, deadline: i64) -> String {
    format!("{PREFIX}{deadline:0WIDTH$}/{hash}")
}

/// On wasm this reads an unimplemented clock unless `time` is built with its
/// `wasm-bindgen` feature, which the manifest enables for that target only.
fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

fn deadline(ttl: Duration, now: i64) -> i64 {
    let expires = now.saturating_add(i64::try_from(ttl.as_secs()).unwrap_or(i64::MAX));
    expires.saturating_add(BUCKET - 1) / BUCKET * BUCKET
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

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

    /// Spans of `now` are walked rather than sampled: whether two reads share a
    /// tag depends on where they fall against the bucket grid, so a pair chosen
    /// by hand proves nothing either way.
    const TTLS: [u64; 4] = [60, 600, 700, 1200];

    #[test]
    fn a_deadline_never_falls_before_the_ttl() {
        for secs in TTLS {
            let ttl = Duration::from_secs(secs);
            let secs = secs.cast_signed();

            for now in 1_700_000_000..=1_700_000_000 + secs {
                assert!(
                    deadline(ttl, now) >= now + secs,
                    "rounding must never expire a blob early: ttl {secs}s at {now}"
                );
            }
        }
    }

    #[test]
    fn re_reads_over_one_ttl_share_a_bounded_number_of_tags() {
        for secs in TTLS {
            let ttl = Duration::from_secs(secs);
            let secs = secs.cast_signed();
            let start = 1_700_000_000;

            let tags = (start..=start + secs)
                .map(|now| deadline(ttl, now))
                .collect::<BTreeSet<_>>();

            assert!(
                i64::try_from(tags.len()).expect("tag count") <= secs / BUCKET + 1,
                "a hash must not accumulate a tag per read: ttl {secs}s left {} tags",
                tags.len()
            );
        }
    }
}
