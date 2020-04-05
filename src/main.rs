#[macro_use]
extern crate diesel;
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
pub mod schema;
pub mod config;

#[actix_rt::main]
async fn main()
{
//    server_main().unwrap_or_else(|_| {
//        println!("Failed to run server_main()");
//    });

    // let session = get_session_by_code("....").await;
    let session = get_access_token().await;

    println!("{:?}", session);
}
