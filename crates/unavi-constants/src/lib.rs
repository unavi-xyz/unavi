use semver::Version;

const WP_PREFIX: &str = "https://wired-protocol.org/v0/";

pub const WP_VERSION: Version = Version::new(0, 1, 0);

pub mod protocols {
    use crate::WP_PREFIX;

    const PROTOCOL_PREFIX: &str = constcat::concat!(WP_PREFIX, "protocols/");

    pub const HOME_WORLD_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "home-world.json");
    pub const WORLD_HOST_PROTOCOL: &str = constcat::concat!(PROTOCOL_PREFIX, "world-host.json");

    pub const HOME_WORLD_DEFINITION: &[u8] =
        include_bytes!("../../../protocol/dwn/protocols/home-world.json");
    pub const WORLD_HOST_DEFINITION: &[u8] =
        include_bytes!("../../../protocol/dwn/protocols/world-host.json");
}

pub mod schemas {
    use crate::WP_PREFIX;

    const SCHEMA_PREFIX: &str = constcat::concat!(WP_PREFIX, "schemas/");

    pub const REMOTE_RECORD_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "remote-record.json");
    pub const WORLD_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "world.json");
}

pub const REMOTE_DWN_URL: &str = "http://localhost:8080";
pub const WORLD_HOST_DID: &str = "did:web:localhost%3A5000";
