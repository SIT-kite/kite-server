
use diesel::prelude::*;
use diesel::pg::PgConnection;

use super::schema;

mod acl;
mod antispam;
pub mod error;
pub mod models;
pub mod manager;


fn establish_connection() -> PgConnection
{
    let database_url = "postgresql://user:password@address:port/database";
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
