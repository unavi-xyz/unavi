use xdid::methods::key::{DidKeyPair, p256::P256KeyPair};
use zeroize::Zeroizing;

use crate::DIRS;

const KEY_FILE: &str = "key.pem";

pub fn get_or_create_key() -> anyhow::Result<P256KeyPair> {
    let dir = DIRS.data_local_dir();

    let key_path = {
        let mut path = dir.to_path_buf();
        path.push(KEY_FILE);
        path
    };

    if key_path.exists() {
        let pem = Zeroizing::new(std::fs::read_to_string(key_path)?);
        let pair = P256KeyPair::from_pkcs8_pem(pem.as_str())?;
        Ok(pair)
    } else {
        let pair = P256KeyPair::generate();
        std::fs::write(&key_path, pair.to_pkcs8_pem()?)?;
        Ok(pair)
    }
}
