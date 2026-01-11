use std::sync::LazyLock;

use native_db::Models;

mod keys;
mod v1;

pub use keys::{BoolKey, DidKey, HashKey};
pub use v1::{
    BlobPin, DepType, Envelope, Record, RecordAclRead, RecordDep, RecordPin, RecordSchema,
    UserQuota,
};

static MODELS: LazyLock<Models> = LazyLock::new(|| {
    let mut models = Models::new();
    models.define::<v1::BlobPin>().expect("BlobPin");
    models.define::<v1::Envelope>().expect("Envelope");
    models.define::<v1::Record>().expect("Record");
    models.define::<v1::RecordAclRead>().expect("RecordAclRead");
    models.define::<v1::RecordDep>().expect("RecordBlobDep");
    models.define::<v1::RecordPin>().expect("RecordPin");
    models.define::<v1::RecordSchema>().expect("RecordSchema");
    models.define::<v1::UserQuota>().expect("UserQuota");
    models
});
