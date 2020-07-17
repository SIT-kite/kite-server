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
mod jwt;
mod models;
mod services;

// Import main function.
use crate::services::server_main;

fn main() {
    server_main().unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
