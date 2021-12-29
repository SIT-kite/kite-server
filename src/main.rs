// Import main function.
use crate::services::server_main;

mod config;
mod services;

#[tokio::main]
async fn main() {
    server_main().await.unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
