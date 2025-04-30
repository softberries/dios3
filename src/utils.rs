use std::path::PathBuf;
use directories::ProjectDirs;

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", "softberries", env!("CARGO_PKG_NAME"))
}

/// Gets the user specified data directory
/// Eventually takes the system default location
pub fn get_data_dir() -> PathBuf {
    let PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    let DATA_FOLDER: Option<PathBuf> = std::env::var(format!("{}_DATA", PROJECT_NAME.clone())).ok().map(PathBuf::from);
    let directory = if let Some(s) = DATA_FOLDER.clone() {
        s
    } else if let Some(proj_dirs) = project_directory() {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        PathBuf::from(".").join(".data")
    };
    directory
}