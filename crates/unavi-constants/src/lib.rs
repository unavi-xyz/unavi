use semver::Version;

pub mod protocols;
pub mod schemas;

const WP_PREFIX: &str = "https://wired-protocol.org/v0/";

pub const WP_VERSION: Version = Version::new(0, 1, 0);

pub const REMOTE_DWN_URL: &str = match option_env!("DWN_URL") {
    Some(url) => url,
    None => "http://localhost:8080",
};

pub const WORLD_HOST_DID: &str = match option_env!("WORLD_DID") {
    Some(did) => did,
    None => "did:web:localhost%3A5000",
};

pub const MODELS_URL: &str = "https://unavi.nyc3.cdn.digitaloceanspaces.com/assets/models/";
