/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use http::request;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use tonic::transport::{Body, Server};
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
mod user;

#[derive(Clone)]
pub struct KiteGrpcServer {
    // Postgres instance.
    db: PgPool,
}

async fn get_db() -> PgPool {
    tracing::info!("Connecting to the main database...");
    let pool = PgPoolOptions::new()
        .max_connections(config::get().db_conn)
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute("SET TIME ZONE 'Asia/Shanghai';").await?;
                Ok(())
            })
        })
        .connect(config::get().db.as_str())
        .await
        .expect("Could not create database pool");

    tracing::info!("DB connected.");
    pool
}

/// Used for gRPC reflection.
fn load_reflection() -> ServerReflectionServer<impl ServerReflection> {
    let file_descriptor = include_bytes!("../../target/compiled-descriptor.bin");

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

    use tower_http::trace::TraceLayer;
    let layer = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_grpc().on_request(|req: &request::Request<Body>, _span: &tracing::Span| {
                tracing::info!("Incoming request: {:?}", req)
            }),
        )
        .into_inner();

    tracing::info!("Listening on {}...", addr);
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
