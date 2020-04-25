extern crate chrono;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;

use futures::TryFutureExt;

use crate::server::server_main;

pub mod user;
pub mod sale;
pub mod sign;
pub mod server;
pub mod config;
pub mod error;
pub mod jwt;


fn main()
{
    server_main().unwrap_or_else(|e| {
        println!("Failed to run server_main(): {}", e);
    });
}
