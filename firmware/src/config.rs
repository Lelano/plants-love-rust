use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub blink_on: bool,
    pub interval_ms: u64,
    pub gpio_pin: u8,
    pub invert: bool,
    // Optional schedule loaded from config: map of day name -> list of (start,end) HHMM
    // Example (TOML):
    // [schedule]
    // Monday = [[900,1700]]
    // Sat = [[800,1200],[1300,1500]]
    pub schedule: Option<HashMap<String, Vec<(u16, u16)>>>,
    // Pin to use for schedule controller (if schedule is provided)
    pub schedule_pin: u8,
    // Moisture sensor calibration values
    pub moisture_dry_value: Option<i16>,
    pub moisture_wet_value: Option<i16>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            blink_on: true,
            interval_ms: 1000,
            gpio_pin: 17,
            invert: false,
            schedule: None,
            schedule_pin: 27,
            moisture_dry_value: None,
            moisture_wet_value: None,
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
        assert!(!d.invert);
        assert!(d.schedule.is_none());
        assert_eq!(d.schedule_pin, 27);
    }
}
