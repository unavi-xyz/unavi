use blake3::Hash;
use iroh_blobs::{
    api::Store,
    HashAndFormat,
};
use n0_future::StreamExt;
use unavi_assets::MANIFEST;

/// Namespace for the pins this crate owns. Deletions are confined to it, so a
/// sweep can never unpin blobs another subsystem holds.
const PREFIX: &str = "assets/";

fn tag(rel_path: &str) -> String {
    format!("{PREFIX}{rel_path}")
}

/// Pins a manifest blob against garbage collection.
///
/// Set before the download rather than after, so no window exists where the
/// content is present and unprotected.
pub async fn hold(store: &Store, rel_path: &str, hash: Hash) -> anyhow::Result<()> {
    store
        .tags()
        .set(tag(rel_path), HashAndFormat::raw(hash.into()))
        .await?;
    Ok(())
}

/// Whether a tag is one this crate owns, and so may be deleted by a sweep.
fn is_ours(name: &str) -> bool {
    name.starts_with(PREFIX)
}

/// Drops pins under [`PREFIX`] that no longer name a manifest asset, letting
/// the store's GC reclaim content the client stopped shipping.
pub async fn sweep(store: &Store) -> anyhow::Result<()> {
    let live = MANIFEST
        .iter()
        .map(|asset| tag(asset.rel_path))
        .collect::<Vec<_>>();

    let tags = store.tags();
    let mut stale = Vec::new();
    let mut listed = tags.list_prefix(PREFIX).await?;

    while let Some(info) = listed.next().await {
        let name = String::from_utf8_lossy(&info?.name.0).into_owned();
        if is_ours(&name) && !live.contains(&name) {
            stale.push(name);
        }
    }

    for name in stale {
        tags.delete(&name).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_are_namespaced() {
        assert_eq!(tag("model/default.vrm"), "assets/model/default.vrm");
    }

    #[test]
    fn a_sweep_only_owns_its_own_namespace() {
        assert!(is_ours(&tag("model/default.vrm")));
        assert!(
            !is_ours("files/model/default.vrm"),
            "hosted content is another subsystem's pin"
        );
        assert!(
            !is_ours("did:key:z123/abc"),
            "a wds blob tag is another subsystem's pin"
        );
    }

    #[test]
    fn manifest_hashes_are_hex_blake3() {
        for asset in MANIFEST {
            let hash = Hash::from_hex(asset.hash).expect("hex hash");
            assert_eq!(hash.to_hex().as_str(), asset.hash, "round-trips");
        }
    }
}
