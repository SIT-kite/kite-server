use std::error::Error;
use std::fs;

use serde::Deserialize;
use toml;

// Look and rename hotpot.toml.example
const DEFAULT_CONFIG_PATH: &str = "hotpot.toml";


#[derive(Debug, Deserialize)]
pub struct Config {
    // Bind address with type "x.x.x.x:port"
    // Usually "0.0.0.0:80"
    pub bind_addr: String,
    // JWT Secret for encrypt.
    pub jwt_secret: String,
    // Miniprogram appid for Wechat interface, apply on mp.weixin.qq.com
    pub wechat_appid: String,
    // Secret for wechat interface.
    pub wechat_secret: String,
    // User for postgresql.
    pub db_user: String,
    // Password.
    pub db_passwd: String,
    // Postgres host.
    pub db_host: String,
    // Postgresql database port at host.
    pub db_port: u16,
    // database name.
    pub db_name: String,
}

lazy_static! {
   pub  static ref CONFIG: Config = load_config(DEFAULT_CONFIG_PATH);
}


// Load global configuration from DEFAULT_CONFIG_PATH at startup.
fn load_config(config_path: &str) -> Config {
    let config_content = fs::read_to_string(config_path);
    return match config_content {
        Ok(config_content) => {
            let toml_result = toml::from_str(config_content.as_str());
            match toml_result {
                Ok(config) => config,
                Err(e) => panic!("Failed to parse {}: {}", DEFAULT_CONFIG_PATH, e),
            }
        }
        Err(e) => {
            panic!("Could not open {}, {}", DEFAULT_CONFIG_PATH, e);
        }
    };
}