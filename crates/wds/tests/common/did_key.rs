use std::sync::Arc;

use wds::{DataStore, Identity, actor::Actor};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

pub async fn generate_actor(store: &DataStore) -> Actor {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    let identity = Arc::new(Identity::new(did.clone(), key));
    let actor = store.local_actor(identity);

    // Set up default quota for the actor.
    let did_str = did.to_string();
    sqlx::query!(
        "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
        did_str
    )
    .execute(store.db())
    .await
    .expect("create quota");

    actor
}
