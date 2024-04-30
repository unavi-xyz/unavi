use dwn::message::descriptor::protocols::ProtocolDefinition;
use semver::Version;

use crate::util::get_protocol_url;

const REGISTRY_PROTOCOL_DEFINITION: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/protocols/world-registry.json");

pub const REGISTRY_PROTOCOL_VERSION: Version = Version::new(0, 0, 1);

pub fn registry_definition() -> ProtocolDefinition {
    serde_json::from_slice(REGISTRY_PROTOCOL_DEFINITION).unwrap()
}

pub fn registry_protocol_url() -> String {
    get_protocol_url(REGISTRY_PROTOCOL_DEFINITION).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition() {
        let definition = registry_definition();
        assert!(!definition.protocol.is_empty())
    }

    #[test]
    fn test_protocol_url() {
        let url = registry_protocol_url();
        assert!(!url.is_empty());
    }
}
