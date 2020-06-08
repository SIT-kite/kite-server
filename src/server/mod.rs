//! The server module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use actix_web::{web, App, HttpResponse, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

use crate::config::CONFIG;

mod middlewares;
// User related interfaces.
mod acl;
mod user;

// TODO: Features
// - HTTP/2 supported
// - HTTPS
// - log to file / database
// The entrance of server is following.
#[actix_rt::main]
pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let manager = ConnectionManager::<PgConnection>::new(&CONFIG.db_string);
    let pool = r2d2::Pool::new(manager).expect("Could not create database pool");

    // Run actix-web server.
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middlewares::auth::Auth)
            .route(
                "/",
                web::get().to(|| HttpResponse::Ok().body("Hello world")),
            )
            .service(user::login)
    })
    .bind(&CONFIG.bind_addr.as_str())?
    .run()
    .await
}
