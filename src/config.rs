use serde::Deserialize;
use std::fs;
use tracing::Level;
use std::env;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub pem_file: String,
    pub ic_url: String,
    pub oc_public_key: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(with = "LevelDef")]
    pub log_level: Level,
}

fn default_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(13457)
}

#[derive(Deserialize)]
#[serde(remote = "Level")]
enum LevelDef {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
} 