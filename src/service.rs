//! The services module is which accepts and processes requests for client and
//! then calls business logic functions. Server controls database as it do
//! some permission check in acl_middleware

mod contact;
mod electricity;
mod notice;
mod weather;

use poem::middleware::AddData;
use poem::{get, listener::TcpListener, EndpointExt, Route, Server};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;

use crate::config::CONFIG;
use crate::middleware::logger::Logger;

/// User Jwt token carried in each request.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct JwtToken {
    /// UID of current user.
    pub uid: i32,
    /// current user role.
    pub is_admin: bool,
}

fn create_route() -> Route {
    use contact::*;
    use electricity::*;
    use notice::*;
    use weather::*;

    let route = Route::new()
        .at("/notice", get(get_notice_list))
        .at("/contact", get(query_all_contacts))
        .at("/weather/:campus", get(get_weather))
        .nest(
            "/electricity",
            Route::new()
                .at("/room/:room", get(query_room_balance))
                .at("/room/:room/rank", get(query_room_consumption_rank))
                .at("/room/:room/bill/days", get(query_room_bills_by_day))
                .at("/room/:room/bill/hours", get(query_room_bills_by_hour)),
        );
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

    // Start weather update daemon
    use crate::model::weather;
    tokio::spawn(weather::weather_daemon(pool.clone()));

    // Run poem services
    let route = create_route();
    let app = route.with(AddData::new(pool)).with(Logger);
    Server::new(TcpListener::bind(&CONFIG.bind))
        .name("kite-server-v2")
        .run(app)
        .await
}
