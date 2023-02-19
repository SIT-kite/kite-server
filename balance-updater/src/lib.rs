/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
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
            if let Err(e) = vaccum::remove_outdated(&db).await {
                tracing::error!("Failed to remove outdated consumption record: {e}");
            }
            i = 0;
        }
        // pull each 20min
        if let Err(e) = pull::pull_balance_list(&db).await {
            tracing::error!("Failed to pull balance list: {e}");
        }
        i += 1;
    }
}
