//! The services module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

mod electricity;
mod notice;

use poem::middleware::AddData;
use poem::{get, listener::TcpListener, EndpointExt, Route, Server};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Executor;

use crate::config::CONFIG;
use crate::service::electricity::{
    query_room_balance, query_room_bills_by_day, query_room_bills_by_hour, query_room_consumption_rank,
};

/// User Jwt token carried in each request.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JwtToken {
    /// UID of current user.
    pub uid: i32,
    /// current user role.
    pub is_admin: bool,
}

fn create_route() -> Route {
    let route = Route::new()
        .at("/notice", get(notice::get_notice_list))
        .at("/electricity/room/:room", get(query_room_balance))
        .at("/electricity/room/:room/rank", get(query_room_consumption_rank))
        .at("/electricity/room/:room/bill/days", get(query_room_bills_by_day))
        .at("/electricity/room/:room/bill/hours", get(query_room_bills_by_hour));
    Route::new().nest("/v2", route)
}

pub async fn server_main() -> std::io::Result<()> {
    // Create database pool.
    let pool = PgPoolOptions::new()
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
    let app = route.with(AddData::new(pool));
    Server::new(TcpListener::bind(&CONFIG.bind))
        .name("kite-server-v2")
        .run(app)
        .await
}
