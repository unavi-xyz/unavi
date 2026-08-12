pub const DEFAULT_AVATAR: &str = "model/default.vrm";
pub const DEFAULT_CHARACTER_ANIMATIONS: &str = "model/animations.glb";

/// A content-addressed file the client fetches over iroh.
///
/// An iroh blob hash is the blake3 hash of the content, so `b3sum <file>`
/// verifies an entry against what a server reports in its `files.json`. The
/// store verifies downloads against the hash, so the manifest needs no size.
#[derive(Debug, Clone, Copy)]
pub struct AssetSpec {
    /// The path the hosting server uses in its files directory.
    pub rel_path: &'static str,
    pub hash:     &'static str,
}

/// The assets the client pulls over iroh. Every entry must be hosted by a
/// reachable unavi-server (or other provider).
pub const MANIFEST: &[AssetSpec] = &[
    AssetSpec {
        rel_path: DEFAULT_AVATAR,
        hash:     "a2f1a48db6cdf369ab510f6a6fb869d107897231b70c4920ad0357e4930c6281",
    },
    AssetSpec {
        rel_path: DEFAULT_CHARACTER_ANIMATIONS,
        hash:     "9fbda809b00ab14e58356721e0c0a92fe88b9234c486a43b9417c4f27555c0c6",
    },
];

/// The manifest entry for a relative path, if any.
#[must_use]
pub fn manifest_entry(rel_path: &str) -> Option<&'static AssetSpec> {
    MANIFEST.iter().find(|asset| asset.rel_path == rel_path)
}

/// The path Bevy loads a manifest asset by, on every platform.
#[must_use]
pub fn asset_path(relative_path: &str) -> String {
    format!("iroh://{relative_path}")
}

#[must_use]
pub fn default_avatar_path() -> String {
    asset_path(DEFAULT_AVATAR)
}

#[must_use]
pub fn default_character_animations_path() -> String {
    asset_path(DEFAULT_CHARACTER_ANIMATIONS)
}
