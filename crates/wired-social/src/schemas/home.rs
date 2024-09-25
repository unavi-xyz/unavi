use serde::{Deserialize, Serialize};

use crate::util::get_schema_id;

use super::common::RecordLink;

const HOME_SCHEMA: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/schemas/home.json");

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Home {
    pub world: RecordLink,
}

pub fn home_schema_url() -> String {
    get_schema_id(HOME_SCHEMA).unwrap()
}

#[cfg(test)]
mod tests {
    use jsonschema::Validator;

    use super::*;

    #[test]
    fn test_schema() {
        let home = Home {
            world: RecordLink {
                did: "did:example:123".to_string(),
                record_id: "abcde".to_string(),
            },
        };

        let serialized = serde_json::to_vec(&home).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(HOME_SCHEMA).unwrap();
        let schema = Validator::new(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }

    #[test]
    fn test_schema_url() {
        let url = home_schema_url();
        assert!(!url.is_empty());
    }
}
