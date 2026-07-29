use std::path::PathBuf;

use directories::ProjectDirs;

// Get application's data folder for savegames
// Linux:   /home/alice/.config/ratagotchi
// Windows: C:\Users\Alice\AppData\Roaming\ratagotchi
// macOS:   /Users/Alice/Library/Application Support/ratagotchi
pub fn get_storage_folder() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("", "", "ratagotchi") {
        let storage_folder = proj_dirs.data_dir();
        if !storage_folder.to_path_buf().exists() {
            std::fs::create_dir_all(storage_folder)?;
        }
        Ok(storage_folder.to_path_buf())
    } else {
        Err("Failed to get directories".to_string())?
    }
}
