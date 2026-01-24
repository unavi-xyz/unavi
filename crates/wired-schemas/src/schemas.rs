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
            let schema: wds_schema::Schema = ron::from_str(RON_STR).expect("valid schema");
            let bytes = schema.to_bytes().expect("serialize schema");
            StaticSchema {
                hash: blake3::hash(&bytes),
                bytes: Bytes::from(bytes),
            }
        });
    };
}

static_schema!(SCHEMA_ACL, "wds/acl.ron");
static_schema!(SCHEMA_RECORD, "wds/record.ron");

static_schema!(SCHEMA_BEACON, "wired/beacon.ron");
static_schema!(SCHEMA_HOME, "wired/home.ron");
static_schema!(SCHEMA_SPACE, "wired/space.ron");
