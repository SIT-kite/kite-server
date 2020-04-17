#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;

use futures::TryFutureExt;

use config::CONFIG;

use crate::server::server_main;

pub mod user;
pub mod server;
pub mod config;
pub mod error;
pub mod jwt;

fn main()
{
    server_main().unwrap_or_else(|_| {
        println!("Failed to run server_main()");
    });
}
