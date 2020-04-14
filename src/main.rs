#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate num_derive;

use futures::TryFutureExt;

use config::CONFIG;
use user::wechat::*;

use crate::server::server_main;

pub mod user;
pub mod server;
pub mod config;

#[actix_rt::main]
async fn main()
{
    server_main().unwrap_or_else(|_| {
        println!("Failed to run server_main()");
    });

}
