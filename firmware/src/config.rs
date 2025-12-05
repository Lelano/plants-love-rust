use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub blink_on: bool,
    pub interval_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            blink_on: true,
            interval_ms: 1000,
        }
    }
}

fn config_path() -> Option<PathBuf> {
    if let Some(pd) = ProjectDirs::from("org", "plants", "plants-love-rust") {
        let dir = pd.config_dir().to_path_buf();
        let _ = fs::create_dir_all(&dir);
        return Some(dir.join("config.toml"));
    }
    None
}

pub fn load_config() -> AppConfig {
    if let Some(path) = config_path() {
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<AppConfig>(&text) {
                return cfg;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(cfg: &AppConfig) -> io::Result<()> {
    if let Some(path) = config_path() {
        let data = toml::to_string_pretty(cfg)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(path, data)?;
    }
    Ok(())
}
