use crate::config::load_config;

mod service;
mod config;

#[tokio::main]
async fn main() {
    config::CONFIG
        .set(config::load_config())
        .expect("Failed to load configuration file, which is kite.toml by default and can be set by KITE_CONFIG.");

    tokio::join!(
        // Run grpc server
        service::grpc_server(),
    );
}