mod cache;
mod config;
mod error;
mod model;
mod service;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    config::initialize();
    cache::initialize();

    tokio::join!(
        // Run grpc server
        service::grpc_server(),
    );
}
