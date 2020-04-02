#[macro_use]
extern crate diesel;

#[macro_use]
extern crate num_derive;

pub mod user;
pub mod server;
pub mod schema;

use crate::server::server_main;
use futures::TryFutureExt;


fn main()
{
    server_main().unwrap_or_else(|_| {
        println!("Failed to run server_main()");
    });
}
