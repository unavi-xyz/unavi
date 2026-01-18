//! Schema types for WDS documents.
//!
//! This module re-exports types from [`loro_schema`] and [`wired_schemas`].

// Re-export schema types from loro-schema.
pub use loro_schema::{Action, Can, Field, Schema, Who, validate_value};

// Re-export static schemas from wired-schemas.
pub use wired_schemas::{
    SCHEMA_ACL, SCHEMA_BEACON, SCHEMA_HOME, SCHEMA_RECORD, SCHEMA_SPACE, StaticSchema,
};

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ron::ser::PrettyConfig;

    use super::*;

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
