use std::sync::{Arc, LazyLock};

use anyhow::Context;
use directories::ProjectDirs;
use dwn::{Actor, Dwn, document_key::DocumentKey, stores::NativeDbStore};
use unavi_constants::REMOTE_DWN_URL;
use xdid::methods::{
    key::{DidKeyPair, PublicKey, p256::P256KeyPair},
    web::reqwest::Url,
};
use zeroize::Zeroizing;

// Use same project dirs as client to share identity.
static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

const KEY_FILE: &str = "key.pem";

fn get_or_create_key() -> anyhow::Result<P256KeyPair> {
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

pub async fn init_actor() -> anyhow::Result<Actor> {
    let pair = get_or_create_key().context("failed to get or create keypair")?;

    let did = pair.public().to_did();
    tracing::info!("Loaded identity: {did}");

    let store = NativeDbStore::new_in_memory().context("failed to open dwn store")?;
    let dwn = Dwn::from(store);

    let mut actor = Actor::new(did, dwn);

    let key = Arc::<DocumentKey>::new(pair.into());
    actor.sign_key = Some(Arc::clone(&key));
    actor.auth_key = Some(key);

    let remote_url = Url::parse(REMOTE_DWN_URL).context("failed to parse remote url")?;
    actor.remote = Some(remote_url.clone());

    tracing::info!("Syncing with remote DWN: {remote_url}");
    actor
        .sync()
        .await
        .context("failed to sync with remote dwn")?;

    Ok(actor)
}
