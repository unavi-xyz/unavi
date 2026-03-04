use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct ScriptPermissions {
    pub wired_agent: bool,
    pub wired_local_agent: bool,
    pub wired_scene: bool,
}

impl Default for ScriptPermissions {
    fn default() -> Self {
        Self {
            wired_agent: false,
            wired_local_agent: false,
            wired_scene: true,
        }
    }
}
