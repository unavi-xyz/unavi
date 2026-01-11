use bevy::prelude::*;
use bevy_vrm::VrmInstance;

use crate::{Avatar, bones::AvatarBones};

pub const DEFAULT_AVATAR: &str = "model/default.vrm";

/// Builder for spawning a visual avatar entity.
#[derive(Default)]
pub struct AvatarSpawner {
    pub vrm_asset: Option<String>,
}

impl AvatarSpawner {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
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
            .unwrap_or(DEFAULT_AVATAR)
            .to_string();
        let vrm_handle = asset_server.load(vrm_path);

        commands
            .spawn((
                Avatar,
                AvatarBones::default(),
                VrmInstance(vrm_handle),
                Transform::default(),
            ))
            .id()
    }
}
