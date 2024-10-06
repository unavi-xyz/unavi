use wired::{dwn::WiredDwn, log::WiredLog, player::WiredPlayer, scene::WiredScene};

mod id;
pub mod wired;

#[derive(Default)]
pub struct ApiData {
    pub wired_dwn: Option<WiredDwn>,
    pub wired_log: Option<WiredLog>,
    pub wired_player: Option<WiredPlayer>,
    pub wired_scene: Option<WiredScene>,
}

#[cfg(test)]
pub mod tests {
    use bevy::prelude::*;

    use crate::{data::ScriptData, env::ScriptEnvBuilder, load::DefaultMaterial};

    pub fn init_test_data() -> (App, ScriptData) {
        let mut app = App::new();

        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Mesh>()
            .init_asset::<StandardMaterial>();

        let default_material = Handle::default();
        app.insert_resource(DefaultMaterial(default_material.clone()));

        let mut env = ScriptEnvBuilder::default();
        env.enable_wired_log("test".to_string());
        env.enable_wired_scene(default_material);

        (app, env.data)
    }
}
