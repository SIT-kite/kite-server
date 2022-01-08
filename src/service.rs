//! The services module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

mod notice;

use poem::{get, listener::TcpListener, Route, Server};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Executor;

use crate::config::CONFIG;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
}

/// User Jwt token carried in each request.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JwtToken {
    /// UID of current user.
    pub uid: i32,
    /// current user role.
    pub is_admin: bool,
}

fn create_route() -> Route {
    Route::new().at("/v2/notice", get(notice::get_recent_notice_list))
}

pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let _pool = PgPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn| {
            Box::pin(async move {
                conn.execute("SET TIME ZONE 'Asia/Shanghai';").await?;
                Ok(())
            })
        })
        .connect(&CONFIG.db.as_ref())
        .await
        .expect("Could not create database pool");

    // tracing_subscriber::fmt::init();

    // Run poem services
    let route = create_route();
    Server::new(TcpListener::bind(&CONFIG.bind))
        .name("kite-server-v2")
        .run(route)
        .await
}
