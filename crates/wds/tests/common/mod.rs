#![allow(dead_code)]

use blake3::Hash;
use loro::LoroDoc;
use rstest::fixture;
use tempfile::{TempDir, tempdir};
use wds::{
    DataStore,
    actor::Actor,
    record::{
        Record,
        acl::Acl,
        envelope::Envelope,
        schema::{SCHEMA_ACL, SCHEMA_RECORD, SCHEMA_STR_ACL, SCHEMA_STR_RECORD, Schema},
    },
    signed_bytes::{Signable, SignedBytes},
    sync::shared::store_envelope,
};
use xdid::{
    core::did::Did,
    methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair},
};

pub struct DataStoreCtx {
    pub store: DataStore,
    pub alice: TestActor,
    pub bob: TestActor,
    _dir: TempDir,
}

/// Test actor with exposed signing key for direct envelope manipulation.
pub struct TestActor {
    pub did: Did,
    pub signing_key: P256KeyPair,
    pub actor: Actor,
}

impl TestActor {
    /// Creates a new record with the given ACL and schemas.
    /// Returns the record ID and the [`LoroDoc`].
    pub async fn create_record(
        &self,
        store: &DataStore,
        acl: Acl,
        schemas: Vec<Hash>,
    ) -> anyhow::Result<(Hash, LoroDoc)> {
        let doc = LoroDoc::new();

        let mut record = Record::new(self.did.clone());
        for schema in schemas {
            record.add_schema(schema);
        }
        record.save(&doc)?;

        acl.save(&doc)?;

        let envelope = Envelope::all_updates(self.did.clone(), &doc)?;
        let signed = envelope.sign(&self.signing_key)?;
        let env_bytes = postcard::to_stdvec(&signed)?;

        let id = record.id()?;
        let id_str = id.to_string();

        // Pin the record first.
        let did_str = self.did.to_string();
        let expires = (time::OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp();
        sqlx::query!(
            "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
            id_str,
            did_str,
            expires
        )
        .execute(store.db())
        .await?;

        // Store the envelope.
        store_envelope(store.db(), store.blobs(), &id_str, &env_bytes).await?;

        Ok((id, doc))
    }

    /// Builds and signs an envelope from a [`LoroDoc`].
    pub fn build_envelope(&self, doc: &LoroDoc) -> anyhow::Result<SignedBytes<Envelope>> {
        let envelope = Envelope::all_updates(self.did.clone(), doc)?;
        envelope.sign(&self.signing_key)
    }

    /// Submits a signed envelope to a record.
    pub async fn submit_envelope(
        &self,
        store: &DataStore,
        record_id: &Hash,
        signed: &SignedBytes<Envelope>,
    ) -> anyhow::Result<()> {
        let env_bytes = postcard::to_stdvec(signed)?;
        let id_str = record_id.to_string();
        store_envelope(store.db(), store.blobs(), &id_str, &env_bytes).await
    }
}

async fn generate_test_actor(store: &DataStore) -> TestActor {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    let actor = store.actor(did.clone(), key.clone());

    // Set up default quota for the actor.
    let did_str = did.to_string();
    sqlx::query!(
        "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000000)",
        did_str
    )
    .execute(store.db())
    .await
    .expect("create quota");

    TestActor {
        did,
        signing_key: key,
        actor,
    }
}

#[fixture]
pub async fn ctx() -> DataStoreCtx {
    let dir = tempdir().expect("tempdir");
    let store = DataStore::new(dir.path())
        .await
        .expect("construct data store");

    // Upload built-in schemas to the blob store.
    for (_hash, bytes) in builtin_schema_bytes() {
        store
            .blobs()
            .blobs()
            .add_slice(&bytes)
            .await
            .expect("add schema");
    }

    let alice = generate_test_actor(&store).await;
    let bob = generate_test_actor(&store).await;

    DataStoreCtx {
        store,
        alice,
        bob,
        _dir: dir,
    }
}

#[must_use]
pub fn builtin_schema_bytes() -> Vec<(Hash, Vec<u8>)> {
    let acl_schema: Schema = ron::from_str(SCHEMA_STR_ACL).expect("valid acl schema");
    let record_schema: Schema = ron::from_str(SCHEMA_STR_RECORD).expect("valid record schema");

    vec![
        (
            *SCHEMA_ACL,
            postcard::to_stdvec(&acl_schema).expect("serialize acl schema"),
        ),
        (
            *SCHEMA_RECORD,
            postcard::to_stdvec(&record_schema).expect("serialize record schema"),
        ),
    ]
}

/// Creates a default ACL with only the given DID having full access.
pub fn acl_only(did: &Did) -> Acl {
    Acl {
        manage: vec![did.clone()],
        write: vec![did.clone()],
        read: vec![],
    }
}

/// Creates an ACL with manage for owner and write for writer.
pub fn acl_with_writer(owner: &Did, writer: &Did) -> Acl {
    Acl {
        manage: vec![owner.clone()],
        write: vec![owner.clone(), writer.clone()],
        read: vec![],
    }
}

/// Creates an ACL with manage for owner and read for reader.
pub fn acl_with_reader(owner: &Did, reader: &Did) -> Acl {
    Acl {
        manage: vec![owner.clone()],
        write: vec![owner.clone()],
        read: vec![reader.clone()],
    }
}
