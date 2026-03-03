use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct ScriptPermissions {
    pub wired_scene: bool,
    pub wired_player: bool,
}

impl Default for ScriptPermissions {
    fn default() -> Self {
        Self {
            wired_scene: true,
            wired_player: false,
        }
    }
}
