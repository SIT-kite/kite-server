use deadpool_postgres::{Config as DeadpoolConfig, Manager as PoolManager, Pool};
use tokio_postgres::NoTls;

use crate::config::CONFIG;

pub fn load_pg_config() -> DeadpoolConfig {
    DeadpoolConfig {
        user: Some(CONFIG.db_user.clone()),
        password: Some(CONFIG.db_passwd.clone()),
        dbname: Some(CONFIG.db_name.clone()),
        host: Some(CONFIG.db_host.clone()),
        port: Some(CONFIG.db_port),
        application_name: Some(String::from("kite-server")),

        ..DeadpoolConfig::default()
    }
}