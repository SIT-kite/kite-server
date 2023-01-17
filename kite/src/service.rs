#[async_trait::async_trait]
pub trait KiteModule {
    async fn run();
}
