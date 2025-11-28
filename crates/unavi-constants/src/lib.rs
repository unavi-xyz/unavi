use semver::Version;

/// Wired Protocol version number.
pub const WP_VERSION: Version = Version::new(0, 1, 0);

pub const REMOTE_DWN_URL: &str = match option_env!("DWN_URL") {
    Some(url) => url,
    None => "http://localhost:8080",
};

pub const SPACE_HOST_DID: &str = match option_env!("SPACE_DID") {
    Some(did) => did,
    None => "did:web:localhost%3A5000",
};

pub const CDN_ASSETS_URL: &str = "https://unavi.nyc3.cdn.digitaloceanspaces.com/assets";

pub const PORTAL_RENDER_LAYER: usize = 5;
