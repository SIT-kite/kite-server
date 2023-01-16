use crate::config::CONFIG;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use tonic::transport::Server;
pub mod gen;

mod ping;

pub struct KiteGrpcServer {
    // Postgres instance.
    db: PgPool,
}

async fn get_db() -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute("SET TIME ZONE 'Asia/Shanghai';").await?;
                Ok(())
            })
        })
        .connect(CONFIG.get().unwrap().db.as_str())
        .await
        .expect("Could not create database pool")
}

pub async fn grpc_server() {
    let addr = "[::1]:50051".parse().unwrap();
    let server = KiteGrpcServer { db: get_db().await };

    let ping = ping::ping_service_server::PingServiceServer::new(server);

    Server::builder().add_service(ping).serve(addr).await.unwrap()
}
