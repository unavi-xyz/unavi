use crate::WP_PREFIX;

const PROTOCOL_PREFIX: &str = constcat::concat!(WP_PREFIX, "protocols/");

pub const HOME_SPACE_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "home-space.json");
pub const SPACE_HOST_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "space-host.json");

pub const HOME_SPACE_DEFINITION: &[u8] =
    include_bytes!("../../../protocol/dwn/protocols/home-space.json");
pub const SPACE_HOST_DEFINITION: &[u8] =
    include_bytes!("../../../protocol/dwn/protocols/space-host.json");
