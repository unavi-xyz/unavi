const ENV_REGISTRY_DID: Option<&str> = option_env!("UNAVI_REGISTRY_DID");
const LOCAL_REGISTRY_DID: &str = "did:web:localhost%3A3000";

pub const fn registry_did() -> &'static str {
    match ENV_REGISTRY_DID {
        Some(value) => value,
        None => LOCAL_REGISTRY_DID,
    }
}
