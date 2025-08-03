use std::ffi::{c_void, OsStr};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::ptr::null;
use anyhow::{anyhow, Result};
use dioxus::logger::tracing::{debug, info, trace};
use lnk::encoding::WINDOWS_1252;
use windows_sys::core::{PCSTR, PSTR};
use windows_sys::Win32::Foundation::{GetLastError, FALSE};
use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
use windows_sys::Win32::System::Threading::{CreateProcessA, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION, STARTUPINFOA};
use crate::loader::App;

/// The place start menu shortcuts are stored (on windows)
const WIN_SHORTCUT_DIR: &'static str = r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\";

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

            if filename.extension().unwrap_or(OsStr::new("")) == "lnk" {
                if let Some(resolved_path) = resolve_shortcut(filename.to_str().unwrap())? {
                    let generated = App::new_from_osstr(&resolved_path, filename.file_name().unwrap());
                    debug!("Resolved entry: {:#?} ",generated);
                    Ok(Some(generated))
                }
                else{
                    Ok(None)
                }
            }
            else{
                Ok(None)
            }
        })
        .map(|x| x.unwrap())
        .flatten()
        .collect()
    )
}

/// Resolve the path of a `.lnk` shortcut. Only works under windows
#[cfg(target_os = "windows")]
fn resolve_shortcut(path: &str) -> Result<Option<String>>{
    let link = lnk::ShellLink::open(path, WINDOWS_1252)?;

    Ok(link.link_target())
}

/// Run an executable
/// 
/// TODO: MAKE THIS SHIT WORK
#[cfg(target_os = "windows")]
pub(crate) fn run_executable(path: PathBuf) -> Result<()> {
    debug!("Path provided:      {:?}", path);
    debug!("Argument provided:  {}", format!("\"{}\"",path.parent().unwrap().to_str().unwrap()));
    debug!("Directory searched: {}", format!("{}",path.file_name().unwrap().to_str().unwrap()));

    // Use expect if .to_str() fails, because it's an exceptional case, and
    // it should be physically impossible for .file_name() to error
    Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg("/D")
        .arg(format!("{}",path.parent().unwrap().to_str().unwrap()))
        .arg(format!("{}",path.file_name().unwrap().to_str().unwrap()))
        .spawn()?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn run_executable(path: PathBuf) -> Result<()> {
    Command::new("sh")
        .arg("-c")
        .arg(path.to_str()?)
        .spawn()?;

    Ok(())
}