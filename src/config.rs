use lazy_static::lazy_static;
use serde::Deserialize;

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

    /* External API */
    /// QWeather.com API key.
    pub qweather_key: String,
}

lazy_static! {
    pub static ref CONFIG: ServerConfig = load_config();
}

fn get_config_path() -> String {
    std::env::var_os("KITE_CONFIG")
        .and_then(|s| Some(s.into_string().unwrap()))
        .unwrap_or(DEFAULT_CONFIG_PATH.to_string())
}

/// Load the global configuration on startup.
fn load_config() -> ServerConfig {
    let path = get_config_path();

    std::fs::read_to_string(&path)
        .map_err(anyhow::Error::new)
        .and_then(|f| toml::from_str(&f).map_err(anyhow::Error::new))
        .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", path, e))
}
