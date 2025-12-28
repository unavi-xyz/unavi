use std::{collections::BTreeMap, sync::LazyLock};

use blake3::Hash;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

macro_rules! static_schema_id {
    ($name:ident) => {
        paste::paste! {
            /// Raw schema string for the builtin schema.
            pub const [<SCHEMA_STR_$name:upper>]: &str = include_str!(concat!("../../../../protocol/schemas/", stringify!($name), ".ron"));
            /// Pre-computed hash of the builtin schema.
            pub static [<SCHEMA_$name:upper>]: LazyLock<Hash> = LazyLock::new(|| {
                let schema: Schema = ron::from_str([<SCHEMA_STR_$name:upper>]).expect("valid schema");
                schema.id().expect("schema id")
            });
        }
    };
}

static_schema_id!(acl);
static_schema_id!(beacon);
static_schema_id!(home);
static_schema_id!(record);
static_schema_id!(space);

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
    /// Serialize to wire format (postcard).
    pub fn to_bytes(&self) -> postcard::Result<Vec<u8>> {
        postcard::to_stdvec(self)
    }

    /// Deserialize from wire format (postcard).
    pub fn from_bytes(bytes: &[u8]) -> postcard::Result<Self> {
        postcard::from_bytes(bytes)
    }

    /// Compute content-addressed ID (blake3 hash of wire bytes).
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
