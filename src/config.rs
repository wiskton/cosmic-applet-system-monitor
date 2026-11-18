use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayFormat {
    UsedSlashTotal,        // Ex: 377G / 460G
    UsedSlashTotalPercent, // Ex: 377G / 460G (86%)
    Compact,               // Ex: 377G/460G
    FreeSlashTotal,        // Ex: Livre 60G / 460G
    PercentOnly,           // Ex: 86%
}

impl Default for DisplayFormat {
    fn default() -> Self {
        DisplayFormat::UsedSlashTotal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RamFormat {
    Percent,               // Ex: 63%
    UsedSlashTotal,        // Ex: 3.2G / 7.5G
    UsedSlashTotalPercent, // Ex: 3.2G / 7.5G (42%)
}

impl Default for RamFormat {
    fn default() -> Self {
        RamFormat::Percent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub show_cpu: bool,
    pub show_ram: bool,
    pub show_disk: bool,
    pub show_net: bool,
    pub disk_label: String,
    pub disk_mount_point: String,
    pub disk_format: DisplayFormat,
    pub ram_format: RamFormat,
    pub show_decimals: bool,
    pub warn_percent: f64,
    pub crit_percent: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            show_cpu: true,
            show_ram: true,
            show_disk: true,
            show_net: true,
            disk_label: "DISK".to_string(),
            disk_mount_point: "/".to_string(),
            disk_format: DisplayFormat::UsedSlashTotal,
            ram_format: RamFormat::Percent,
            show_decimals: false,
            warn_percent: 75.0,
            crit_percent: 90.0,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/cosmic-applet-disk-usage/config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&data) {
                return cfg;
            }
        }
        let default = Self::default();
        let _ = default.save();
        default
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
