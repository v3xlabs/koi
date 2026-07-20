use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Terminal,
    Dark,
    Midnight,
    Light,
    Paper,
}

impl Default for Theme {
    fn default() -> Self {
        Self::Dark
    }
}

impl Theme {
    pub const ALL: [Self; 5] = [
        Self::Terminal,
        Self::Dark,
        Self::Midnight,
        Self::Light,
        Self::Paper,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal",
            Self::Dark => "Dark",
            Self::Midnight => "Midnight",
            Self::Light => "Light",
            Self::Paper => "Paper",
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|theme| *theme == self)
            .unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    #[serde(default = "default_display_currency")]
    pub display_currency: String,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_colored_assets")]
    pub colored_assets: bool,
    #[serde(default)]
    pub collapsed_groups: Vec<Option<u64>>,
}

fn default_display_currency() -> String {
    "fiat:usd".to_string()
}

fn default_colored_assets() -> bool {
    true
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            display_currency: default_display_currency(),
            theme: Theme::default(),
            colored_assets: default_colored_assets(),
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
