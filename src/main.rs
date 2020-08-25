#![allow(dead_code)]

extern crate chrono;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;
extern crate log;

mod config;
mod error;
mod ipset;
mod jwt;
mod models;
mod services;
mod task;

// Import main function.
use crate::services::server_main;

use crate::task::Host;
use futures::TryFutureExt;
use std::sync::Arc;

#[actix_rt::main]
async fn main() {
    let host = Host {
        agents: Arc::new(Default::default()),
    };
    tokio::spawn(async move {
        host.websocket_main().await.unwrap_or_else(|e| {
            println!("Failed to run websocket host: {}", e);
        });
    });

    server_main()
        .unwrap_or_else(|e| {
            println!("Failed to run server_main(): {}", e);
        })
        .await;
}
