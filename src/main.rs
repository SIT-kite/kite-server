#![allow(dead_code)]

extern crate chrono;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;
extern crate log;

mod bridge;
mod config;
mod error;
mod ipset;
mod jwt;
mod models;
mod services;

// Import main function.
use crate::services::server_main;
use futures::TryFutureExt;

#[actix_web::main]
async fn main() {
    server_main()
        .unwrap_or_else(|e| {
            println!("Failed to run server_main(): {}", e);
        })
        .await;
}
