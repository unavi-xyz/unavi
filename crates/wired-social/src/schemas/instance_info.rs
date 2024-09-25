use serde::{Deserialize, Serialize};

use crate::util::get_schema_id;

const INSTANCE_INFO_SCHEMA: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/schemas/instance-info.json");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub url: String,
    #[serde(rename = "numPlayers", skip_serializing_if = "Option::is_none")]
    pub num_players: Option<usize>,
    #[serde(rename = "maxPlayers", skip_serializing_if = "Option::is_none")]
    pub max_players: Option<usize>,
}

pub fn instance_info_schema_url() -> String {
    get_schema_id(INSTANCE_INFO_SCHEMA).unwrap()
}

#[cfg(test)]
mod tests {
    use jsonschema::Validator;

    use super::*;

    #[test]
    fn test_schema() {
        let instance = InstanceInfo {
            url: "https://example.com".to_string(),
            num_players: Some(0),
            max_players: Some(100),
        };

        let serialized = serde_json::to_vec(&instance).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(INSTANCE_INFO_SCHEMA).unwrap();
        let schema = Validator::new(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }

    #[test]
    fn test_schema_url() {
        let url = instance_info_schema_url();
        assert!(!url.is_empty());
    }
}
