use std::{path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub fn assets_dir() -> PathBuf {
    DIRS.data_local_dir().join("assets")
}

pub fn copy_assets_to_project_dir(paths: &[&str]) {
    let assets = assets_dir();
    for path in paths {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../unavi-client/assets")
            .join(path);
        let dst = assets.join(path);
        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::copy(&src, &dst) {
            eprintln!("failed to copy {path}: {e}");
        }
    }
}
