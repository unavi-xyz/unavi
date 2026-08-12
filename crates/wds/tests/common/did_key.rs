use std::sync::Arc;

use rusqlite::params;
use wds::{
    DataStore,
    actor::Actor,
    identity::Identity,
};
use xdid::methods::key::keys::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

pub async fn generate_actor(store: &DataStore) -> Actor {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    let identity = Arc::new(Identity::new(did.clone(), key));
    let actor = store.local_actor(identity);

    let did_str = did.to_string();
    store
        .db()
        .call(move |conn| {
            conn.execute(
                "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
                params![&did_str],
            )?;
            Ok(())
        })
        .await
        .expect("create quota");

    actor
}
