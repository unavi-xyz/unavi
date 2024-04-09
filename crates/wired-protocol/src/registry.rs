use dwn::message::descriptor::protocols::ProtocolDefinition;
use semver::Version;
use serde::{Deserialize, Serialize};

const REGISTRY_PROTOCOL_DEFINITION: &[u8] =
    include_bytes!("../../../wired-protocol/social/dwn/protocols/world-registry.json");

pub const REGISTRY_PROTOCOL_VERSION: Version = Version::new(0, 0, 1);

pub fn registry_definition() -> ProtocolDefinition {
    serde_json::from_slice(REGISTRY_PROTOCOL_DEFINITION).unwrap()
}

pub fn registry_world_schema_url() -> String {
    registry_definition()
        .types
        .get("world")
        .unwrap()
        .schema
        .clone()
        .unwrap()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use jsonschema::JSONSchema;

    use super::*;

    #[test]
    fn test_registry_definition() {
        let definition = registry_definition();
        assert!(!definition.protocol.is_empty())
    }

    #[test]
    fn test_registry_world_schema() {
        let schema = registry_world_schema_url();
        assert!(!schema.is_empty())
    }

    #[test]
    fn world_matches_schema() {
        const WORLD_SCHEMA: &[u8] =
            include_bytes!("../../../wired-protocol/social/dwn/schemas/world.json");

        let world = World {
            name: Some("my_name".to_string()),
            description: Some("my_description".to_string()),
            image: Some("my_image".to_string()),
            tags: Some(vec!["tag_1".to_string(), "tag_2".to_string()]),
        };

        let serialized = serde_json::to_vec(&world).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(WORLD_SCHEMA).unwrap();
        let schema = JSONSchema::compile(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }
}
