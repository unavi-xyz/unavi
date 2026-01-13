//! Platform-agnostic asset path resolution for UNAVI.
//!
//! On native builds, returns relative paths for Bevy's file-based asset loader.
//! On web builds, returns full CDN URLs for Bevy's [`WebAssetPlugin`].

/// CDN base URL for web assets.
pub const CDN_URL: &str = "https://unavi.nyc3.cdn.digitaloceanspaces.com/assets";

/// Default avatar model path (relative).
pub const DEFAULT_AVATAR: &str = "model/default.vrm";

/// Default animations file path (relative).
pub const DEFAULT_ANIMATIONS: &str = "model/animations.glb";

/// Returns the asset path appropriate for the current platform.
///
/// - **Native**: Returns the relative path as-is (Bevy loads from local assets dir).
/// - **Web**: Returns `{CDN_URL}/{relative_path}` for HTTP loading.
#[must_use]
pub fn asset_path(relative_path: &str) -> String {
    #[cfg(target_family = "wasm")]
    {
        format!("{CDN_URL}/{relative_path}")
    }

    #[cfg(not(target_family = "wasm"))]
    {
        relative_path.to_string()
    }
}

#[must_use]
pub fn default_avatar_path() -> String {
    asset_path(DEFAULT_AVATAR)
}

#[must_use]
pub fn default_animations_path() -> String {
    asset_path(DEFAULT_ANIMATIONS)
}
