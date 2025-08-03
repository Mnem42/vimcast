use crate::{apps_json_path, update_apps_json};
use anyhow::Result;

pub fn initialize_config() -> std::io::Result<()> {
    let path = apps_json_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Only create a blank file if it doesn't exist
    if !path.exists() {
        std::fs::write(&path, "[]")?;
    }

    Ok(())
}

pub fn try_update_apps_json() -> Result<()> {
    update_apps_json();
    Ok(())
}
