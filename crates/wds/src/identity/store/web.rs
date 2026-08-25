//! Browser-backed identity storage.
//!
//! The PEM is held in local storage, which any script on the origin can read:
//! an injected script is an identity theft. It is used anyway because signing
//! needs the raw scalar, which a non-extractable `CryptoKey` would never hand
//! back.

use std::str::FromStr;

use iroh_docs::NamespaceId;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

use crate::identity::store::DeviceSeed;

const KEY_ITEM: &str = "unavi.identity.v1";
const SEED_ITEM: &str = "unavi.device.v1";
const NAMESPACE_ITEM_PREFIX: &str = "unavi.ns.";

pub fn load_or_create() -> anyhow::Result<P256KeyPair> {
    let storage = local_storage()?;

    if let Ok(Some(pem)) = storage.get_item(KEY_ITEM) {
        return P256KeyPair::from_pkcs8_pem(Zeroizing::new(pem).as_str());
    }

    let pair = P256KeyPair::generate();
    storage
        .set_item(KEY_ITEM, pair.to_pkcs8_pem()?.as_str())
        .map_err(|_| anyhow::anyhow!("could not write the identity key to local storage"))?;
    Ok(pair)
}

/// Removing [`SEED_ITEM`] is how a device rotates: the next load writes a new
/// seed, and with it a new endpoint id and author id.
pub fn load_or_create_seed() -> anyhow::Result<DeviceSeed> {
    let storage = local_storage()?;

    if let Ok(Some(hex)) = storage.get_item(SEED_ITEM)
        && let Some(bytes) = decode_seed(&hex)
    {
        return Ok(DeviceSeed::from_bytes(bytes));
    }

    let seed = DeviceSeed::generate();
    storage
        .set_item(SEED_ITEM, &encode_seed(seed.as_bytes()))
        .map_err(|_| anyhow::anyhow!("could not write the device seed to local storage"))?;
    Ok(seed)
}

fn namespace_item(label: &str) -> String {
    format!("{NAMESPACE_ITEM_PREFIX}{label}")
}

pub fn load_namespace(label: &str) -> anyhow::Result<Option<NamespaceId>> {
    let Ok(Some(text)) = local_storage()?.get_item(&namespace_item(label)) else {
        return Ok(None);
    };
    Ok(NamespaceId::from_str(text.trim()).ok())
}

pub fn save_namespace(label: &str, ns: NamespaceId) -> anyhow::Result<()> {
    local_storage()?
        .set_item(&namespace_item(label), &ns.to_string())
        .map_err(|_| anyhow::anyhow!("could not write the namespace to local storage"))
}

fn encode_seed(bytes: &[u8; 32]) -> String {
    bytes.iter().fold(String::with_capacity(64), |mut out, b| {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn decode_seed(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

fn local_storage() -> anyhow::Result<web_sys::Storage> {
    web_sys::window()
        .ok_or_else(|| anyhow::anyhow!("no window"))?
        .local_storage()
        .map_err(|_| anyhow::anyhow!("local storage is blocked"))?
        .ok_or_else(|| anyhow::anyhow!("no local storage"))
}
