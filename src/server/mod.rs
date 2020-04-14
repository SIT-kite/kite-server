//! The server module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

use actix_web::{App, HttpResponse, HttpServer, web};
use deadpool_postgres::{Config as DeadpoolConfig, Manager as PoolManager, Pool};
use tokio_postgres::NoTls;

use crate::config::CONFIG;

mod middlewares;
// User related interfaces.
mod user;
mod db;



// TODO: Features
// - HTTP/2 supported
// - HTTPS
// - log to file / database
// The entrance of server is following.
#[actix_rt::main]
pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let cfg = db::load_pg_config();
    let pool = cfg.create_pool(NoTls).expect("Failed to create pool.");

    // Run actix-web server.
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .wrap(middlewares::auth::Auth)
            .route("/", web::get().to(|| HttpResponse::Ok().body("Hello world")))
            .service(user::login)
    })
        .bind(&CONFIG.bind_addr.as_str())?
        .run()
        .await
}