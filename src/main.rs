extern crate chrono;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;

// Import main function.
use crate::server::server_main;

pub mod config;
pub mod error;
pub mod freshman;
pub mod sale;
pub mod schema;
pub mod server;
pub mod sign;
pub mod user;

fn main() {
    server_main().unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
