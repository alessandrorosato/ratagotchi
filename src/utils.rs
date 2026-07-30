use std::path::PathBuf;

use directories::ProjectDirs;
use log::warn;

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

pub fn human_friendly_time_delta(time_delta: i64) -> String {
    let days = time_delta / 86400;
    let remaining_seconds = time_delta % 86400;
    let hours = remaining_seconds / 3600;
    let remaining_seconds = remaining_seconds % 3600;
    let minutes = remaining_seconds / 60;
    let seconds = remaining_seconds % 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days > 1 { "s" } else { "" }));
    }
    if hours > 0 {
        parts.push(format!(
            "{} hour{}",
            hours,
            if hours > 1 { "s" } else { "" }
        ));
    }
    if minutes > 0 {
        parts.push(format!(
            "{} minute{}",
            minutes,
            if minutes > 1 { "s" } else { "" }
        ));
    }
    if seconds > 0 && parts.is_empty() {
        parts.push(format!(
            "{} second{}",
            seconds,
            if seconds > 1 { "s" } else { "" }
        ));
    }

    match parts.len() {
        0 => "0 seconds".to_string(),
        1 => parts[0].clone(),
        _ => {
            if let Some(last) = parts.pop() {
                let rest = parts.join(", ");
                format!("{} and {}", rest, last)
            } else {
                warn!("Couldn't print out time. parts: {:?}", parts);
                "Some time ago...".to_string()
            }
        }
    }
}
