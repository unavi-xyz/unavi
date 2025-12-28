pub const REMOTE_WDS_URL: &str = match option_env!("WDS_URL") {
    Some(url) => url,
    None => "http://localhost:8080",
};

pub const CDN_ASSETS_URL: &str = "https://unavi.nyc3.cdn.digitaloceanspaces.com/assets";

pub const PORTAL_RENDER_LAYER: usize = 5;
