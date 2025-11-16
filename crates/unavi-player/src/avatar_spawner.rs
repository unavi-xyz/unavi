use bevy::prelude::*;
use bevy_vrm::{VrmBundle, VrmInstance};
use constcat::concat;

use crate::{PlayerAvatar, bones::AvatarBones};

const DEFAULT_AVATAR_URL: &str = concat!(unavi_constants::CDN_ASSETS_URL, "/models/default.vrm");

/// Builder for spawning a visual avatar entity.
#[derive(Default)]
pub struct AvatarSpawner {
    pub vrm_asset: Option<String>,
}

impl AvatarSpawner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vrm(mut self, vrm_asset: impl Into<String>) -> Self {
        self.vrm_asset = Some(vrm_asset.into());
        self
    }

    /// Spawns a visual avatar entity.
    /// Returns the avatar entity ID.
    pub fn spawn(&self, commands: &mut Commands, asset_server: &AssetServer) -> Entity {
        let vrm_path = self
            .vrm_asset
            .as_deref()
            .unwrap_or(DEFAULT_AVATAR_URL)
            .to_string();
        let vrm_handle = asset_server.load(vrm_path);

        commands
            .spawn((
                PlayerAvatar,
                AvatarBones::default(),
                VrmBundle {
                    vrm: VrmInstance(vrm_handle),
                    ..Default::default()
                },
                Transform::default(),
            ))
            .id()
    }
}
