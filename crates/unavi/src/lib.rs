use std::sync::LazyLock;

use bevy::prelude::*;
use directories::ProjectDirs;
use unavi_script::LoadScriptAsset;

pub mod icon;
pub mod scene;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi").expect("project dirs");
    std::fs::create_dir_all(dirs.data_dir()).expect("data dir");
    dirs
});

pub fn add_scripts(mut events: EventWriter<LoadScriptAsset>) {
    events.write(LoadScriptAsset {
        namespace: "unavi",
        package: "system",
    });
    events.write(LoadScriptAsset {
        namespace: "unavi",
        package: "ui",
    });
}
