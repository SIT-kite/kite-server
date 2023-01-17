use kite::cache;
use kite::config;
use kite::service::KiteModule;

mod error;
mod model;
mod service;

pub struct ServerV3 {}

#[async_trait::async_trait]
impl KiteModule for ServerV3 {
    async fn run() {
        service::grpc_server().await
    }
}
