use semver::Version;

use crate::util::get_protocol_url;

const WORLD_HOST_PROTOCOL_DEFINITION: &[u8] =
    include_bytes!("../../../../wired-protocol/social/dwn/protocols/world-host.json");

pub const WORLD_HOST_PROTOCOL_VERSION: Version = Version::new(0, 0, 1);

// pub fn world_host_definition() -> ProtocolDefinition {
//     serde_json::from_slice(WORLD_HOST_PROTOCOL_DEFINITION).unwrap()
// }

pub fn world_host_protocol_url() -> String {
    get_protocol_url(WORLD_HOST_PROTOCOL_DEFINITION).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_definition() {
    //     let definition = world_host_definition();
    //     assert!(!definition.protocol.is_empty())
    // }

    #[test]
    fn test_protocol_url() {
        let url = world_host_protocol_url();
        assert!(!url.is_empty());
    }
}
