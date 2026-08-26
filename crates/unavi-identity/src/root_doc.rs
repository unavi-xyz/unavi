use std::str::FromStr;

use iroh_docs::NamespaceId;
use wds::DataStore;

#[cfg(target_family = "wasm")]
const ITEM: &str = "unavi.root-doc.v1";
#[cfg(not(target_family = "wasm"))]
const FILE: &str = "root-doc";

/// Opens this node's root document, minting it on first use.
///
/// The namespace id is recorded next to the identity it belongs to, so a
/// restart reopens the document users, scripts, and trust checks already
/// reference. A recorded id whose capability the docs store no longer holds
/// means that store was lost; the document is unrecoverable either way, so a
/// fresh one is minted and the pointer replaced.
///
/// `persistent` is false for an in-memory session, which mints fresh every
/// run and leaves nothing behind.
pub async fn open_or_mint(store: &DataStore, persistent: bool) -> anyhow::Result<NamespaceId> {
    cfg_select! {
        target_family = "wasm" => open_or_mint_web(store).await,
        _ => open_or_mint_fs(store, persistent).await,
    }
}

async fn mint(store: &DataStore) -> anyhow::Result<NamespaceId> {
    Ok(store.docs().api().create().await?.id())
}

#[cfg(not(target_family = "wasm"))]
async fn open_or_mint_fs(store: &DataStore, persistent: bool) -> anyhow::Result<NamespaceId> {
    let path = unavi_util::dirs::data_local_dir().to_path_buf().join(FILE);

    if persistent
        && let Ok(text) = std::fs::read_to_string(&path)
        && let Ok(ns) = NamespaceId::from_str(text.trim())
        && store.docs().api().open(ns).await?.is_some()
    {
        return Ok(ns);
    }

    let ns = mint(store).await?;
    if persistent {
        std::fs::write(path, ns.to_string())?;
    }
    Ok(ns)
}

#[cfg(target_family = "wasm")]
async fn open_or_mint_web(store: &DataStore) -> anyhow::Result<NamespaceId> {
    let storage = web_sys::window()
        .ok_or_else(|| anyhow::anyhow!("no window"))?
        .local_storage()
        .map_err(|_| anyhow::anyhow!("local storage is blocked"))?
        .ok_or_else(|| anyhow::anyhow!("no local storage"))?;

    if let Ok(Some(text)) = storage.get_item(ITEM)
        && let Ok(ns) = NamespaceId::from_str(text.trim())
        && store.docs().api().open(ns).await?.is_some()
    {
        return Ok(ns);
    }

    let ns = mint(store).await?;
    storage
        .set_item(ITEM, &ns.to_string())
        .map_err(|_| anyhow::anyhow!("could not write the root doc id to local storage"))?;
    Ok(ns)
}
