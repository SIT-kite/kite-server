use std::time::Duration;

use anyhow::Result;
use tokio::time;

use kite::service::KiteModule;

mod cache;
mod pull;
mod vaccum;

pub struct BalanceUpdater {}

#[async_trait::async_trait]
impl KiteModule for BalanceUpdater {
    async fn run() {
        daemon().await.expect("BalanceUpdater exited.");
    }
}

async fn daemon() -> Result<()> {
    let db = kite::get_db();

    let duration = Duration::from_secs(60 * 20);
    let mut interval = time::interval(duration);

    // Because it's a little hard to write async-callback in rust, I use one loop to handle these two
    // events. Maybe it can be rewrite in future.
    let mut i = 0;
    loop {
        interval.tick().await;
        if i % 72 == 0 {
            // 72 * 20min = one day
            let _ = vaccum::remove_unused(&db).await;
            i = 0;
        }
        // pull each 20min
        let _ = pull::pull_balance_list(&db).await;
        i += 1;
    }
}
