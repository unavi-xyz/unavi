use directories::ProjectDirs;

pub fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("xyz", "unavi", "unavi-app").expect("Failed to get project dirs.")
}
