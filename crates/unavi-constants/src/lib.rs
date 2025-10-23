use semver::Version;

pub mod protocols;
pub mod schemas;

const WP_PREFIX: &str = "https://wired-protocol.org/v0/";

pub const WP_VERSION: Version = Version::new(0, 1, 0);

pub const REMOTE_DWN_URL: &str = "http://localhost:8080";
pub const WORLD_HOST_DID: &str = "did:web:localhost%3A5000";
