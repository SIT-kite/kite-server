// Import main function.
use crate::service::server_main;

mod config;
mod error;
mod model;
mod response;
mod service;

#[tokio::main]
async fn main() {
    server_main().await.unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
