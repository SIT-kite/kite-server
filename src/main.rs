#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;

use futures::TryFutureExt;
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

use crate::server::server_main;

pub mod user;
pub mod server;
pub mod schema;
pub mod config;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    uid: i32,
    role: String,
}


fn main()
{
    server_main().unwrap_or_else(|_| {
        println!("Failed to run server_main()");
    });
}
