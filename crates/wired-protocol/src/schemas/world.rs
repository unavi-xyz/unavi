use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<Host>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Host {
    pub did: String,
    pub record: String,
}

#[cfg(test)]
mod tests {
    use jsonschema::JSONSchema;

    use super::*;

    const SCHEMA: &[u8] =
        include_bytes!("../../../../wired-protocol/social/dwn/schemas/world.json");

    #[test]
    fn test_schema() {
        let world = World {
            name: Some("my_name".to_string()),
            description: Some("my_description".to_string()),
            tags: Some(vec!["tag_1".to_string(), "tag_2".to_string()]),
            host: Some(Host {
                did: "did:example:123".to_string(),
                record: "abcd".to_string(),
            }),
        };

        let serialized = serde_json::to_vec(&world).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(SCHEMA).unwrap();
        let schema = JSONSchema::compile(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }
}
