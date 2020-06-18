extern crate chrono;
#[macro_use]
extern crate diesel;
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
pub mod sale;
pub mod schema;
pub mod server;
pub mod sign;
pub mod user;

// Imporrt library.
// #[warn(unused_imports)]
// use futures::TryFutureExt;

fn main() {
    server_main().unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
