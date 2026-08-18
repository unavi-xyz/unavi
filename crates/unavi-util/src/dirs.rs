use std::{
    path::Path,
    sync::LazyLock,
};

use directories::ProjectDirs;

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    std::fs::create_dir_all(dirs.config_dir()).expect("config dir");
    dirs
});

/// The app's data directory, shared by every client-side crate.
#[must_use]
pub fn data_local_dir() -> &'static Path {
    DIRS.data_local_dir()
}

/// Where files meant to be opened in an editor live, as opposed to state the
/// app keeps for itself.
#[must_use]
pub fn config_dir() -> &'static Path {
    DIRS.config_dir()
}
