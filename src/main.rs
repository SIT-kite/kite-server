#![allow(dead_code)]

extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate log;
#[macro_use]
extern crate num_derive;

use futures::TryFutureExt;

// Import main function.
use crate::services::server_main;

mod bridge;
mod config;
mod error;
mod ipset;
mod jwt;
mod models;
mod services;

#[actix_web::main]
async fn main() {
    server_main()
        .unwrap_or_else(|e| {
            println!("Failed to run server_main(): {}", e);
        })
        .await;
}
