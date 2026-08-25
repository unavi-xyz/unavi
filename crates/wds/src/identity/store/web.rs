//! Browser-backed identity storage.
//!
//! The PEM is held in local storage, which any script on the origin can read:
//! an injected script is an identity theft. It is used anyway because signing
//! and key derivation both need the raw scalar, which a non-extractable
//! `CryptoKey` would never hand back. Revisit if the P-256 key stops being the
//! derivation root.

use xdid::methods::key::keys::{
    DidKeyPair,
    p256::P256KeyPair,
};
use zeroize::Zeroizing;

const KEY_ITEM: &str = "unavi.identity.v1";

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

fn local_storage() -> anyhow::Result<web_sys::Storage> {
    web_sys::window()
        .ok_or_else(|| anyhow::anyhow!("no window"))?
        .local_storage()
        .map_err(|_| anyhow::anyhow!("local storage is blocked"))?
        .ok_or_else(|| anyhow::anyhow!("no local storage"))
}
