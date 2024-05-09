const ENV_WORLD_HOST_DID: Option<&str> = option_env!("UNAVI_WORLD_HOST_DID");
const LOCAL_WORLD_HOST_DID: &str = "did:web:localhost%3A3001";

pub const fn world_host_did() -> &'static str {
    match ENV_WORLD_HOST_DID {
        Some(value) => value,
        None => LOCAL_WORLD_HOST_DID,
    }
}
