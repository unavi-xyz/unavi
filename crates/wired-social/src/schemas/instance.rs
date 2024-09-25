use serde::{Deserialize, Serialize};

use crate::util::get_schema_id;

use super::common::RecordLink;

const INSTANCE_SCHEMA: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/schemas/instance.json");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Instance {
    pub world: RecordLink,
}

pub fn instance_schema_url() -> String {
    get_schema_id(INSTANCE_SCHEMA).unwrap()
}

#[cfg(test)]
mod tests {
    use jsonschema::Validator;

    use super::*;

    #[test]
    fn test_schema() {
        let instance = Instance {
            world: RecordLink {
                did: "did:example:123".to_string(),
                record_id: "abcde".to_string(),
            },
        };

        let serialized = serde_json::to_vec(&instance).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(INSTANCE_SCHEMA).unwrap();
        let schema = Validator::new(&schema).unwrap();

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
