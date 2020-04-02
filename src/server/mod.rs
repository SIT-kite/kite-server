//! The server module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use actix_web::{App, HttpServer, web, HttpResponse};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

// Access control for requests.
mod acl_middleware;
// User related interfaces.
mod user_service;
// Error structures and handlers
mod error;

// TODO: Features
// - HTTP/2 supported
// - HTTPS
// - log to file / database
// The entrance of server is following.
#[actix_rt::main]
pub async fn server_main() -> std::io::Result<()> {
    // TODO: Read configuration from file.
    // Config database.
    let database_url = "postgresql://user:password@address:port/database";
    let db_conn = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(db_conn)
        .expect("Fail to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .route("/", web::get().to(|| HttpResponse::Ok().body("Hello world")))
            .service(user_service::login)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}