//! Browser-backed identity storage.
//!
//! The PEM lives in local storage, readable by any script on the origin,
//! because signing needs the raw scalar, which a non-extractable `CryptoKey`
//! never yields back.

use iroh::SecretKey;
use unavi_store::local::web::storage as local_storage;
use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

const KEY_ITEM: &str = "unavi.identity";
const ENDPOINT_ITEM: &str = "unavi.endpoint";

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

/// Removing [`ENDPOINT_ITEM`] is how a device rotates: the next load writes a
/// new key, and with it a new `EndpointId` and author id.
pub fn load_or_create_endpoint() -> anyhow::Result<SecretKey> {
    let storage = local_storage()?;

    if let Ok(Some(hex)) = storage.get_item(ENDPOINT_ITEM)
        && let Some(bytes) = decode_key(&hex)
    {
        return Ok(SecretKey::from_bytes(&bytes));
    }

    let key = SecretKey::generate();
    storage
        .set_item(ENDPOINT_ITEM, &encode_key(&key.to_bytes()))
        .map_err(|_| anyhow::anyhow!("could not write the endpoint key to local storage"))?;
    Ok(key)
}

fn encode_key(bytes: &[u8; 32]) -> String {
    bytes.iter().fold(String::with_capacity(64), |mut out, b| {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn decode_key(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(i * 2..i * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}
