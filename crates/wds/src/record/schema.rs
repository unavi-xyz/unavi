use std::{collections::BTreeMap, sync::LazyLock};

use blake3::Hash;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// A statically-defined schema with precomputed hash and bytes.
pub struct StaticSchema {
    pub hash: Hash,
    pub bytes: Bytes,
}

macro_rules! static_schema {
    ($name:ident) => {
        paste::paste! {
            pub static [<SCHEMA_$name:upper>]: LazyLock<StaticSchema> = LazyLock::new(|| {
                const RON_STR: &str = include_str!(
                    concat!("../../../../protocol/schemas/", stringify!($name), ".ron")
                );
                let schema: Schema = ron::from_str(RON_STR).expect("valid schema");
                let bytes = schema.to_bytes().expect("serialize schema");
                StaticSchema {
                    hash: blake3::hash(&bytes),
                    bytes: Bytes::from(bytes),
                }
            });
        }
    };
}

static_schema!(acl);
static_schema!(beacon);
static_schema!(home);
static_schema!(record);
static_schema!(space);

/// Schema defining how to process a Loro document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    id: SmolStr,
    version: u32,
    /// The targeted Loro container.
    container: SmolStr,
    /// Allowed layout of the data.
    layout: Field,
}

impl Schema {
    /// # Errors
    ///
    /// Errors if postcard could not serialize the struct.
    pub fn to_bytes(&self) -> postcard::Result<Vec<u8>> {
        postcard::to_stdvec(self)
    }

    /// # Errors
    ///
    /// Errors if postcard could not deserialize the struct.
    pub fn from_bytes(bytes: &[u8]) -> postcard::Result<Self> {
        postcard::from_bytes(bytes)
    }

    /// Compute content-addressed ID.
    ///
    /// # Errors
    ///
    /// Errors if postcard could not serialize the struct.
    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = self.to_bytes()?;
        Ok(blake3::hash(&bytes))
    }

    #[must_use]
    pub fn container(&self) -> &str {
        &self.container
    }

    #[must_use]
    pub const fn layout(&self) -> &Field {
        &self.layout
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Field {
    Restricted {
        actions: Vec<Action>,
        value: Box<Self>,
    },
    List(Box<Self>),
    Map(BTreeMap<SmolStr, Box<Self>>),
    Any,
    Bool,
    F64,
    I64,
    String,
    Binary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub who: Who,
    pub can: Vec<Can>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Who {
    Anyone,
    Path(SmolStr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Can {
    Create,
    Delete,
    Update,
}

#[cfg(test)]
mod tests {
    use ron::ser::PrettyConfig;

    use super::*;

    #[test]
    fn test_parse_schemas() {
        let schemas_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../protocol/schemas");
        println!("Reading {}", schemas_path.display());

        for entry in std::fs::read_dir(schemas_path).expect("read schemas dir") {
            let path = entry.expect("read entry").path();
            println!("Reading {}", path.display());

            let schema_str = std::fs::read_to_string(&path).expect("read entry file");

            let schema: Schema = ron::from_str(&schema_str).expect("from_str");

            let out =
                ron::ser::to_string_pretty(&schema, PrettyConfig::default()).expect("to_string");

            std::fs::write(path, out).expect("write pretty formatted");
        }
    }
}
