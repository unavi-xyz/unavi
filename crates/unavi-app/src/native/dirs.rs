use directories::ProjectDirs;

pub fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("xyz", "unavi", "unavi-app").expect("Failed to get project dirs.")
}

pub fn init_dirs() -> Result<(), std::io::Error> {
    let dirs = get_project_dirs();
    std::fs::create_dir_all(dirs.data_dir())?;
    Ok(())
}
