use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RecordLink {
    pub record: String,
}

#[cfg(test)]
mod tests {
    use jsonschema::JSONSchema;

    use super::*;

    const SCHEMA: &[u8] =
        include_bytes!("../../../../wired-protocol/social/dwn/schemas/record-link.json");

    #[test]
    fn test_schema() {
        let record_link = RecordLink {
            record: "abcd".to_string(),
        };

        let serialized = serde_json::to_vec(&record_link).unwrap();
        let deserialized = serde_json::from_slice(&serialized).unwrap();

        let schema = serde_json::from_slice(SCHEMA).unwrap();
        let schema = JSONSchema::compile(&schema).unwrap();

        if schema.validate(&deserialized).is_err() {
            panic!("Failed to validate");
        };
    }
}
