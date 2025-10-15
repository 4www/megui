use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub website: String,
    pub resume: String,
    pub artworks: String,
    pub repository: String,
    #[serde(default = "default_theme")]
    pub default_theme: String,
}

fn default_theme() -> String {
    "auto".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
}

impl Config {
    pub fn load() -> Self {
        const CONFIG_TOML: &str = include_str!("../config.toml");
        toml::from_str(CONFIG_TOML).expect("Failed to parse config.toml")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::load()
    }
}
