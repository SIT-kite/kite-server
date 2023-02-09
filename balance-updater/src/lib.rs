use anyhow::Result;
use tokio::time;

use kite::service::KiteModule;

mod cache;
mod pull;

pub struct BalanceUpdater {}

#[async_trait::async_trait]
impl KiteModule for BalanceUpdater {
    async fn run() {
        daemon().await.expect("BalanceUpdater exited.");
    }
}

async fn daemon() -> Result<()> {
    let db = kite::get_db();
    let mut interval = time::interval(time::Duration::from_secs(60 * 30));

    loop {
        interval.tick().await;
        pull::pull_balance_list(db).await?;
    }
}
