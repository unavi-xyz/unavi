use serde::{Deserialize, Serialize};

use super::util::schema_id;

const INSTANCE_SCHEMA: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/schemas/instance.json");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Instance {
    pub world: RecordLink,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecordLink {
    pub did: String,
    pub record: String,
}

pub fn instance_schema_url() -> String {
    schema_id(INSTANCE_SCHEMA).unwrap()
}

#[cfg(test)]
mod tests {
    use jsonschema::JSONSchema;

    use super::*;

    #[test]
    fn test_schema() {
        let instance = Instance {
            world: RecordLink {
                did: "did:example:123".to_string(),
                record: "abcde".to_string(),
            },
        };

        let serialized = serde_json::to_vec(&instance).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(INSTANCE_SCHEMA).unwrap();
        let schema = JSONSchema::compile(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }

    #[test]
    fn test_schema_url() {
        let url = instance_schema_url();
        assert!(!url.is_empty());
    }
}
