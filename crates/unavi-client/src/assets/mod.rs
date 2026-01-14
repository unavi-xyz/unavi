use std::{path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;

pub mod copy;
pub mod download;

pub static DIRS: LazyLock<directories::ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub fn assets_dir() -> PathBuf {
    DIRS.data_local_dir().join("assets")
}
