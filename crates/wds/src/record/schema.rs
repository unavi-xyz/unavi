use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Schema defining how to process a Loro document.
#[derive(Debug, Serialize, Deserialize)]
pub struct Schema {
    id: SmolStr,
    version: u32,
    /// The targeted Loro container.
    container: SmolStr,
    /// Allowed layout of the data.
    layout: Field,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Action {
    who: Who,
    can: Vec<Can>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Who {
    Anyone,
    Path(SmolStr),
}

#[derive(Debug, Serialize, Deserialize)]
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
