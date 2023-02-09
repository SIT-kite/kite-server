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

use anyhow::Result;
use once_cell::sync::OnceCell;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};

use crate::config;

static DB: OnceCell<PgPool> = OnceCell::new();

pub async fn initialize_db() -> Result<()> {
    tracing::info!("Connecting to the main database...");
    let pool = PgPoolOptions::new()
        .max_connections(config::get().db_conn)
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute("SET TIME ZONE 'Asia/Shanghai';").await?;
                Ok(())
            })
        })
        .connect(config::get().db.as_str())
        .await?;

    tracing::info!("DB connected.");
    DB.set(pool).expect("Don't initialize db more than once.");
    Ok(())
}

pub fn get_db() -> &'static PgPool {
    DB.get().expect("DB is not initialized!!!")
}