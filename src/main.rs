#![allow(non_snake_case)]

extern crate chrono;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate log;

pub mod attachment;
pub mod config;
pub mod error;
pub mod freshman;
mod jwt;
mod motto;
pub mod sale;
pub mod schema;
pub mod server;
pub mod sign;
pub mod user;
pub mod wechat;

// Import main function.
use crate::server::server_main;

fn main() {
    server_main().unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
