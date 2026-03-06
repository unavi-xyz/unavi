pub const CDN_URL: &str = "https://unavi.nyc3.cdn.digitaloceanspaces.com/assets";

pub const DEFAULT_AVATAR: &str = "model/default.vrm";

pub const DEFAULT_CHARACTER_ANIMATIONS: &str = "model/animations.glb";
pub const DEFAULT_MENU_ANIMATION: &str = "model/animation-menu.glb";

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
pub fn default_character_animations_path() -> String {
    asset_path(DEFAULT_CHARACTER_ANIMATIONS)
}

#[must_use]
pub fn default_menu_animation_path() -> String {
    asset_path(DEFAULT_MENU_ANIMATION)
}
