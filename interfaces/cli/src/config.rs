use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_display_currency")]
    pub display_currency: String,
    #[serde(default)]
    pub collapsed_groups: Vec<Option<u64>>,
}

fn default_display_currency() -> String {
    "fiat:usd".to_string()
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            display_currency: default_display_currency(),
            collapsed_groups: Vec::new(),
        }
    }
}

impl TuiConfig {
    pub fn load() -> Self {
        config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(contents) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, contents);
        }
    }
}

fn config_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("koi").join("tui.json"))
}
