use std::sync::LazyLock;

use blake3::Hash;
use bytes::Bytes;

pub struct StaticSchema {
    pub hash: Hash,
    pub bytes: Bytes,
}

macro_rules! static_schema {
    ($name:ident, $path:literal) => {
        pub static $name: LazyLock<StaticSchema> = LazyLock::new(|| {
            const RON_STR: &str = include_str!(concat!("../../../protocol/schemas/", $path));
            let schema: loro_schema::Schema = ron::from_str(RON_STR).expect("valid schema");
            let bytes = schema.to_bytes().expect("serialize schema");
            StaticSchema {
                hash: blake3::hash(&bytes),
                bytes: Bytes::from(bytes),
            }
        });
    };
}

static_schema!(SCHEMA_ACL, "acl.ron");
static_schema!(SCHEMA_BEACON, "beacon.ron");
static_schema!(SCHEMA_HOME, "home.ron");
static_schema!(SCHEMA_RECORD, "record.ron");
static_schema!(SCHEMA_SPACE, "space.ron");
