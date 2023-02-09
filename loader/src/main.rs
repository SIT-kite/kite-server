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

use kite::cache;
use kite::config;
use kite::db;
use kite::service::KiteModule;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();
    tracing::info!("Starting...");

    config::initialize();
    cache::initialize();

    db::initialize_db().await.expect("Could not create database pool.");
    captcha::async_init().await.expect("Failed to init captcha service.");

    tokio::join! {
        service_v3::ServerV3::run(),
        balance_updater::BalanceUpdater::run(),
    };
}
