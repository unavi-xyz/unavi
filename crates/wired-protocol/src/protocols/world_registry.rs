use dwn::message::descriptor::protocols::ProtocolDefinition;
use semver::Version;

const REGISTRY_PROTOCOL_DEFINITION: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/protocols/world-registry.json");

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

#[cfg(test)]
mod tests {
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
}
