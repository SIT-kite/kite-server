use kite::cache;
use kite::config;
use kite::service::KiteModule;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    tracing::info!("Starting...");

    config::initialize();
    cache::initialize();

    tokio::join! {
        service_v3::ServerV3::run()
    };
}
