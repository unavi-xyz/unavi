use dwn::message::descriptor::protocols::ProtocolDefinition;
use semver::Version;

const REGISTRY_PROTOCOL_DEFINITION: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/protocols/world-registry.json");

pub const REGISTRY_PROTOCOL_VERSION: Version = Version::new(0, 0, 1);

pub fn registry_definition() -> ProtocolDefinition {
    serde_json::from_slice(REGISTRY_PROTOCOL_DEFINITION).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition() {
        let definition = registry_definition();
        assert!(!definition.protocol.is_empty())
    }
}
