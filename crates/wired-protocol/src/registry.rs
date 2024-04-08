use dwn::message::descriptor::protocols::ProtocolDefinition;
use semver::Version;
use serde_json::Value;

pub const REGISTRY_PROTOCOL_DEFINITION: &str =
    include_str!("../../../wired-protocol/social/dwn/protocols/world-registry.json");

pub const REGISTRY_PROTOCOL_VERSION: Version = Version::new(0, 0, 1);

pub fn registry_definition() -> ProtocolDefinition {
    serde_json::from_str(REGISTRY_PROTOCOL_DEFINITION).unwrap()
}

pub fn registry_protocol() -> String {
    let value: Value = serde_json::from_str(REGISTRY_PROTOCOL_DEFINITION).unwrap();
    value["protocol"].as_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_definition() {
        let _ = registry_definition();
    }

    #[test]
    fn test_registry_protocol() {
        let protocol = registry_protocol();
        assert!(!protocol.is_empty());
    }
}
