use lazy_static::lazy_static;
use serde::Deserialize;
use std::fs;

// Look and rename kite.example.toml
const DEFAULT_CONFIG_PATH: &str = "kite.toml";

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

lazy_static! {
    pub static ref CONFIG: ServerConfig = load_config(DEFAULT_CONFIG_PATH)
        .unwrap_or_else(|e| { panic!("Failed to parse {}: {}", DEFAULT_CONFIG_PATH, e) });
}

/// Load the global configuration from DEFAULT_CONFIG_PATH on the startup.
fn load_config(config_path: &str) -> Result<ServerConfig, anyhow::Error> {
    let config_content = fs::read_to_string(config_path)?;
    let config = toml::from_str(config_content.as_str())?;

    Ok(config)
}
