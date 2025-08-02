use std::fs;
use std::path::PathBuf;
use anyhow::anyhow;
use crate::loader::App;

/// The place start menu shortcuts are stored (on windows)
const WIN_SHORTCUT_DIR: &'static str = r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs";


#[cfg(target_os = "macos")]
fn search_app_dirs() -> anyhow::Result<Vec<crate::loader::App>> {
    let app_dirs: Vec<PathBuf> = vec![
        PathBuf::from("/Applications"),
        dirs::home_dir().unwrap_or_default().join("Applications"),
    ];

    let apps = Vec::new();

    for dir in app_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                {
                    if path.extension().map(|ext| ext == "app").unwrap_or(false) {
                        if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                            let exists = apps.iter().any(|a| a.name.eq_ignore_ascii_case(name));
                            if !exists {
                                apps.push(crate::loader::App {
                                    name: name.to_string(),
                                    command: format!("open -a \"{}\"", name),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    return apps;
}


#[cfg(target_os = "windows")]
pub(crate) fn  search_app_dirs() -> anyhow::Result<Vec<crate::loader::App>> {
    Ok(fs::read_dir(WIN_SHORTCUT_DIR)?
        .map(|x| -> anyhow::Result<Option<crate::loader::App>> {
            let filename = x?.path();

            if let Some(filename) = filename.to_str() {
                if  filename.ends_with(".lnk") {
                    Ok(Some(App::new(filename, filename)))
                }
                else{
                    Ok(None)
                }
            }
            else{
                Err(anyhow!("Error finding stuff"))
            }
        })
        .map(|x| x.unwrap())
        .flatten()
        .collect()
    )
}