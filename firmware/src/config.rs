use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub blink_on: bool,
    pub interval_ms: u64,
    pub gpio_pin: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            blink_on: true,
            interval_ms: 1000,
            gpio_pin: 17,
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
            .map_err(io::Error::other)?;
        fs::write(path, data)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let d = AppConfig::default();
        assert!(d.blink_on);
        assert_eq!(d.interval_ms, 1000);
        assert_eq!(d.gpio_pin, 17);
    }
}
