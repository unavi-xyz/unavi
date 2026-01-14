use std::sync::Arc;

use rusqlite::params;
use wds::{DataStore, actor::Actor, identity::Identity};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

pub async fn generate_actor(store: &DataStore) -> Actor {
    generate_actor_with_identity(store).await.0
}

/// Generates a did:key actor and returns both the actor and identity.
pub async fn generate_actor_with_identity(store: &DataStore) -> (Actor, Arc<Identity>) {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    let identity = Arc::new(Identity::new(did.clone(), key));
    let actor = store.local_actor(Arc::clone(&identity));

    // Set up default quota for the actor.
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

    (actor, identity)
}
