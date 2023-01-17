use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use tonic::transport::Server;
use tonic_reflection::server::{ServerReflection, ServerReflectionServer};

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

/// Used for gRPC reflection.
fn load_reflection() -> ServerReflectionServer<impl ServerReflection> {
    let file_descriptor = include_bytes!("../target/compiled-descriptor.bin");

    tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(file_descriptor)
        .build()
        .unwrap()
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

    use tower_http::trace::{DefaultOnRequest, TraceLayer};
    use tracing::Level;
    let layer = tower::ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc().on_request(DefaultOnRequest::new().level(Level::INFO)))
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(load_reflection())
        .add_service(ping)
        .add_service(badge)
        .add_service(balance)
        .add_service(board)
        .add_service(classroom_browser)
        .serve(addr)
        .await
        .unwrap()
}
