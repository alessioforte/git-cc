use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub scopes: Vec<String>,
}

impl Settings {
    pub fn new() -> Self {
        Settings { scopes: vec![] }
    }

    pub fn save(&self) {
        let path = get_config_path();
        println!("Saving settings to: {:?}", path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Could not create config directory");
        }
        let file_content = serde_json::to_string(self).expect("Could not serialize settings");
        std::fs::write(path, file_content).expect("Could not write settings file");
    }
}

pub fn get_settings() -> Settings {
    let path = get_config_path();
    if !path.exists() {
        return Settings::new();
    }
    let file_content = std::fs::read_to_string(path).expect("Could not read settings file");
    serde_json::from_str(&file_content).expect("Could not parse settings file")
}

pub fn get_config_path() -> PathBuf {
    let git_dir = find_git_root().expect("Could not find git root directory");
    git_dir.join("cc").join("settings.json")
}

pub fn find_git_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        let git_dir = dir.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Some(git_dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

pub fn get_temp_commit_msq_path() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    path.push("git_commit.txt");
    path
}
