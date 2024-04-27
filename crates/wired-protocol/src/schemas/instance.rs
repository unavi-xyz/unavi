use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Instance {
    pub url: String,
    #[serde(rename = "numPlayers", skip_serializing_if = "Option::is_none")]
    pub num_players: Option<usize>,
    #[serde(rename = "maxPlayers", skip_serializing_if = "Option::is_none")]
    pub max_players: Option<usize>,
}

#[cfg(test)]
mod tests {
    use jsonschema::JSONSchema;

    use super::*;

    const SCHEMA: &[u8] =
        include_bytes!("../../../../wired-protocol/social/dwn/schemas/instance.json");

    #[test]
    fn test_schema() {
        let instance = Instance {
            url: "https://example.com".to_string(),
            num_players: Some(0),
            max_players: Some(100),
        };

        let serialized = serde_json::to_vec(&instance).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(SCHEMA).unwrap();
        let schema = JSONSchema::compile(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }
}
