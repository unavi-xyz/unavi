//! Schema types for Loro document validation.

use std::collections::BTreeMap;

use blake3::Hash;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Schema defining how to validate a Loro container.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    id: SmolStr,
    version: u32,
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

    /// # Errors
    ///
    /// Errors if postcard could not serialize the struct.
    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = self.to_bytes()?;
        Ok(blake3::hash(&bytes))
    }

    #[must_use]
    pub const fn layout(&self) -> &Field {
        &self.layout
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Field {
    Any,
    Binary,
    Bool,
    F64,
    I64,
    List(Box<Self>),
    /// Homogeneous map: any string keys, all values match inner type.
    Map(Box<Self>),
    MovableList(Box<Self>),
    Optional(Box<Self>),
    Restricted {
        actions: Vec<Action>,
        value: Box<Self>,
    },
    String,
    /// Fixed keys with heterogeneous types per field.
    Struct(BTreeMap<SmolStr, Box<Self>>),
    /// Each node's metadata must match inner type.
    Tree(Box<Self>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub who: Who,
    pub can: Vec<Can>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Who {
    Anyone,
    /// Path in the document containing authorized DIDs.
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
    use std::path::Path;

    use ron::ser::PrettyConfig;

    use super::Schema;

    fn parse_schemas_in_path(path: &Path) {
        if path.is_dir() {
            for item in path.read_dir().expect("read dir") {
                let item = item.expect("read entry").path();

                parse_schemas_in_path(&item);
            }
        } else {
            println!("Reading {}", path.display());

            let schema_str = std::fs::read_to_string(path).expect("read entry file");

            let schema: Schema = ron::from_str(&schema_str).expect("from_str");

            let out =
                ron::ser::to_string_pretty(&schema, PrettyConfig::default()).expect("to_string");

            std::fs::write(path, out).expect("write pretty formatted");
        }
    }

    #[test]
    fn test_parse_schemas() {
        let schemas_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../protocol/schemas");
        println!("Reading {}", schemas_path.display());

        parse_schemas_in_path(&schemas_path);
    }
}
