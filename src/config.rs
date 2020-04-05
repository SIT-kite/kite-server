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
    pub db_string: String,
    pub jwt_secret: String,
    pub wechat_appid: String,
    pub wechat_secret: String,
}

lazy_static! {
   pub  static ref CONFIG: Config = load_config(DEFAULT_CONFIG_PATH);
}

pub fn load_config(config_path: &str) -> Config {
    let config_content = fs::read_to_string(config_path);
    if let Ok(content) = config_content {
        let toml_result = toml::from_str(content.as_str());
        if let Ok(config) = toml_result {
            return config;
        }
        panic!("Failed to parse config, err = {:?}\nContent: {:#?}", toml_result, content);
    }
    panic!("Failed to open config: {:#?}", config_content);
}