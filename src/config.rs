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

#[allow(dead_code)]
impl DisplayFormat {
    pub fn label(&self) -> &'static str {
        match self {
            DisplayFormat::UsedSlashTotal => "377G / 460G",
            DisplayFormat::UsedSlashTotalPercent => "377G / 460G (86%)",
            DisplayFormat::Compact => "377G/460G",
            DisplayFormat::FreeSlashTotal => "Livre: 60G / 460G",
            DisplayFormat::PercentOnly => "86%",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            DisplayFormat::UsedSlashTotal => DisplayFormat::UsedSlashTotalPercent,
            DisplayFormat::UsedSlashTotalPercent => DisplayFormat::Compact,
            DisplayFormat::Compact => DisplayFormat::FreeSlashTotal,
            DisplayFormat::FreeSlashTotal => DisplayFormat::PercentOnly,
            DisplayFormat::PercentOnly => DisplayFormat::UsedSlashTotal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub label: String,
    pub mount_point: String,
    pub display_format: DisplayFormat,
    pub show_decimals: bool,
    pub refresh_interval_secs: u64,
    pub warn_percent: f64,
    pub crit_percent: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            label: "DISK".to_string(),
            mount_point: "/".to_string(),
            display_format: DisplayFormat::UsedSlashTotal,
            show_decimals: false,
            refresh_interval_secs: 3,
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
