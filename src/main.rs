// Import main function.
use crate::service::server_main;

mod config;
mod error;
mod middleware;
mod model;
mod portal;
mod response;
mod service;

#[tokio::main]
async fn main() {
    /// Load configuration.
    config::CONFIG.set(config::load_config());

    server_main().await.unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
