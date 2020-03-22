#[macro_use]
extern crate diesel;

pub mod user;
pub mod server;
pub mod schema;

use crate::user::models::{Verification};
use crate::server::server_main;


fn main()
{
//    if let Ok(bindings) = get_verifications_by_uid(1) {
//        for each_approach in bindings {
//            println!("{:?}", each_approach);
//        }
//    }
    server_main();
}
