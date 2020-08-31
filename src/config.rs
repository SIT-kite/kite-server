use serde::Deserialize;
use std::fs;

// Look and rename kite.example.toml
const DEFAULT_CONFIG_PATH: &str = "kite.toml";

#[derive(Deserialize)]
pub struct Config {
    /// Server config
    pub server: ServerConfig,
    /// Wechat config
    pub wechat: WechatConfig,
    /// Host config. Used to config the communication with agents.
    pub host: HostConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    /// Bind address with type "x.x.x.x:port"
    /// Usually "0.0.0.0:443"
    pub bind: String,
    /// JWT Secret for encrypt.
    pub secret: String,
    /// Database for postgresql.
    pub db: String,
    /// Attachment directory.
    pub attachment: String,
}

#[derive(Deserialize)]
pub struct WechatConfig {
    /// Micro-app appid for Wechat interface, apply on mp.weixin.qq.com
    pub appid: String,
    /// Secret for wechat interface.
    pub secret: String,
}

#[derive(Deserialize)]
pub struct HostConfig {
    /// Bind address with the format "x.x.x.x:port",
    /// for accepting connections from agents
    pub bind: String,
    /// Max agent count.
    pub max: u8,
}

lazy_static! {
    pub static ref CONFIG: Config = load_config(DEFAULT_CONFIG_PATH)
        .unwrap_or_else(|e| { panic!("Failed to parse {}: {}", DEFAULT_CONFIG_PATH, e) });
}

/// Load the global configuration from DEFAULT_CONFIG_PATH on the startup.
fn load_config(config_path: &str) -> Result<Config, anyhow::Error> {
    let config_content = fs::read_to_string(config_path)?;
    let config = toml::from_str(config_content.as_str())?;

    Ok(config)
}
