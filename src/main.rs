mod cache;
mod config;
mod error;
mod model;
mod service;

#[tokio::main]
async fn main() {
    config::initialize();
    cache::initialize();

    tokio::join!(
        // Run grpc server
        service::grpc_server(),
    );
}
