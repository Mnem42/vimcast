use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use dioxus::desktop::tao::rwh_05::RawWindowHandle::Win32;
use crate::search::RadixNode;
use anyhow::{anyhow, Result};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct App {
    name: String,
    command: String,
}

impl App{
    pub fn new(name: &str, command: &str) -> Self{
        Self {
            name: name.to_string(),
            command: command.to_string(),
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("vimcast")
}

pub fn apps_json_path() -> PathBuf {
    config_path().join("apps.json")
}

pub fn load(db: &mut RadixNode) {
    let path = apps_json_path();
    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to read apps.json: {err}");
            return;
        }
    };

    let apps: Vec<App> = match serde_json::from_str(&data) {
        Ok(apps) => apps,
        Err(err) => {
            eprintln!("Failed to parse apps.json: {err}");
            return;
        }
    };

    for app in apps {
        db.insert(&app.name);
    }
}

pub fn launch(app_name: String) {
    let path = apps_json_path();
    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Failed to read apps.json: {err}");
            return;
        }
    };

    let apps: Vec<App> = match serde_json::from_str(&data) {
        Ok(apps) => apps,
        Err(err) => {
            eprintln!("Failed to parse apps.json: {err}");
            return;
        }
    };

    if let Some(app) = apps.iter().find(|a| a.name.eq_ignore_ascii_case(&app_name)) {
        if let Err(err) = std::process::Command::new("sh")
            .arg("-c")
            .arg(&app.command)
            .spawn()
        {
            eprintln!("Failed to execute command: {err}");
        }
    } else {
        eprintln!("App '{}' not found in apps.json", app_name);
    }
}

pub fn update_apps_json_with_installed_apps() {
    let config_dir = config_path();
    let path = apps_json_path();

    if let Err(err) = fs::create_dir_all(&config_dir) {
        eprintln!("Failed to create config directory: {err}");
        return;
    }

    let apps = search_app_dirs().unwrap();

    match File::create(&path) {
        Ok(file) => {
            let writer = BufWriter::new(file);
            if let Err(err) = serde_json::to_writer_pretty(writer, &apps) {
                eprintln!("Failed to write apps.json: {err}");
            }
        }
        Err(err) => {
            eprintln!("Failed to create apps.json: {err}");
        }
    }
}