use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub username: String,
    pub auth_token: String,
}

impl Config {
    pub fn from_file(path: &Path) -> Config {
        let contents = std::fs::read_to_string(path).expect("Failed to read config file");
        toml::from_str(&contents).expect("Invalid config file format")
    }
}
