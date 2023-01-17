/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use once_cell::sync::OnceCell;
use serde::Deserialize;

static CONFIG: OnceCell<ServerConfig> = OnceCell::new();

// Look and rename kite.example.toml
const DEFAULT_CONFIG_PATH: &str = "kite.toml";

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    /// Bind address with type "x.x.x.x:port"
    /// Usually "0.0.0.0:443"
    pub bind: String,
    /// JWT Secret for encrypt.
    pub secret: String,
    /// Database for postgresql.
    pub db: String,
    /// Max db conn
    pub db_conn: u32,

    /// Cache db
    pub cache: Option<String>,

    /* External API */
    /// QWeather.com API key.
    pub qweather_key: String,
}

fn get_config_path() -> String {
    std::env::var_os("KITE_CONFIG")
        .and_then(|s| Some(s.into_string().unwrap()))
        .unwrap_or(DEFAULT_CONFIG_PATH.to_string())
}

/// Load the global configuration on startup.
pub fn load_config() -> ServerConfig {
    let path = get_config_path();

    std::fs::read_to_string(&path)
        .map_err(anyhow::Error::new)
        .and_then(|f| toml::from_str(&f).map_err(anyhow::Error::new))
        .unwrap_or_else(|e| panic!("Failed to parse {:?}: {}", path, e))
}

pub fn initialize() {
    tracing::debug!("Loading configuration...");
    CONFIG
        .set(load_config())
        .expect("Failed to load configuration file, which is kite.toml by default and can be set by KITE_CONFIG.");
}

pub fn get() -> &'static ServerConfig {
    CONFIG.get().expect("Config not initialized but you want to use.")
}
