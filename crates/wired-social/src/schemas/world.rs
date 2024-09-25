use serde::{Deserialize, Serialize};

use crate::util::get_schema_id;

const WORLD_SCHEMA: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/schemas/world.json");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

pub fn world_schema_url() -> String {
    get_schema_id(WORLD_SCHEMA).unwrap()
}

#[cfg(test)]
mod tests {
    use jsonschema::Validator;

    use super::*;

    #[test]
    fn test_schema() {
        let world = World {
            name: Some("my_name".to_string()),
            description: Some("my_description".to_string()),
            tags: Some(vec!["tag_1".to_string(), "tag_2".to_string()]),
            host: Some("did:example:123".to_string()),
        };

        let serialized = serde_json::to_vec(&world).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(WORLD_SCHEMA).unwrap();
        let schema = Validator::new(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }

    #[test]
    fn test_schema_url() {
        let url = world_schema_url();
        assert!(!url.is_empty());
    }
}
