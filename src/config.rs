use std::error::Error;
use std::fs;

use serde::Deserialize;
use toml;

// Example
// bind_addr = "127.0.0.1:80"
// db_string = "postgresql://xxxxx:port/database"
// jwt_string = "secret"

const DEFAULT_CONFIG_PATH: &str = "hotpot.toml";


#[derive(Debug, Deserialize)]
pub struct Config {
    pub bind_addr: Option<String>,
    pub db_string: Option<String>,
    pub jwt_secret: Option<String>,
}

lazy_static! {
   pub  static ref CONFIG: Config = load_config(DEFAULT_CONFIG_PATH);
}

pub fn load_config(config_path: &str) -> Config {
    let config_content = fs::read_to_string(config_path);
    if let Ok(content) = config_content {
        if let Ok(config) = toml::from_str(content.as_str()) {
            return config;
        }
        panic!("Failed to parse config: {:#?}", content);
    }
    panic!("Failed to open config: {:#?}", config_content);
}