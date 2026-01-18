use std::sync::LazyLock;

use blake3::Hash;
use bytes::Bytes;

/// A statically-defined schema with precomputed hash and bytes.
pub struct StaticSchema {
    /// Content-addressed hash of the schema.
    pub hash: Hash,
    /// Serialized schema bytes.
    pub bytes: Bytes,
}

/// Create a static schema from a RON file in protocol/schemas/.
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

// Static schemas loaded at compile time from protocol/schemas/.
static_schema!(SCHEMA_ACL, "acl.ron");
static_schema!(SCHEMA_BEACON, "beacon.ron");
static_schema!(SCHEMA_HOME, "home.ron");
static_schema!(SCHEMA_RECORD, "record.ron");
static_schema!(SCHEMA_SPACE, "space.ron");
