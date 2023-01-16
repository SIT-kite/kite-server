mod config;
mod error;
mod model;
mod service;

#[tokio::main]
async fn main() {
    config::initialize();

    tokio::join!(
        // Run grpc server
        service::grpc_server(),
    );
}
