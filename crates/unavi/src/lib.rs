use std::sync::LazyLock;

use directories::ProjectDirs;

pub mod icon;
pub mod scene;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi").expect("project dirs");

    std::fs::create_dir_all(dirs.data_dir()).expect("data dir");

    dirs
});
