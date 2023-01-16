use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use tonic::transport::Server;

use crate::config;

pub mod auth;
pub mod gen;

mod badge;
mod balance;
mod board;
mod classroom_browser;
mod ping;
mod template;

#[derive(Clone)]
pub struct KiteGrpcServer {
    // Postgres instance.
    db: PgPool,
}

async fn get_db() -> PgPool {
    PgPoolOptions::new()
        .max_connections(config::get().db_conn)
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute("SET TIME ZONE 'Asia/Shanghai';").await?;
                Ok(())
            })
        })
        .connect(config::get().db.as_str())
        .await
        .expect("Could not create database pool")
}

pub async fn grpc_server() {
    let addr = config::get().bind.parse().unwrap();
    let server = KiteGrpcServer { db: get_db().await };

    let ping = ping::gen::ping_service_server::PingServiceServer::new(server.clone());
    let badge = badge::gen::badge_service_server::BadgeServiceServer::new(server.clone());
    let balance = balance::gen::balance_service_server::BalanceServiceServer::new(server.clone());
    let board = board::gen::board_service_server::BoardServiceServer::new(server.clone());
    let classroom_browser =
        classroom_browser::gen::classroom_browser_service_server::ClassroomBrowserServiceServer::new(server.clone());

    Server::builder()
        .add_service(ping)
        .add_service(badge)
        .add_service(balance)
        .add_service(board)
        .add_service(classroom_browser)
        .serve(addr)
        .await
        .unwrap()
}
