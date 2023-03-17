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

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use chrono::Local;
use once_cell::sync::OnceCell;
use serde::de::Error;
use serde::Deserialize;
use sqlx::PgPool;
use tokio::time::Instant;

use super::cache::clear_cache;

#[derive(Deserialize)]
struct RawBalance {
    #[serde(rename(deserialize = "RoomName"), deserialize_with = "str2i32")]
    room: i32,
    #[serde(rename(deserialize = "Balance"), deserialize_with = "str2float")]
    total: f32,
}

type RoomNumber = i32;

const INVALID_ROOM_NUMBER: RoomNumber = 0;

fn str2i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // Some room numbers are start with "-" and used for testing, we will ignore them.
    if let Ok(n) = s.parse::<RoomNumber>() {
        if get_valid_room_set().contains(&n) {
            return Ok(n);
        }
    }
    Ok(INVALID_ROOM_NUMBER)
}

fn str2float<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f32>().map_err(Error::custom)
}

static VALID_ROOM_SET: OnceCell<HashSet<RoomNumber>> = OnceCell::new();

async fn init_valid_room_set(db: &PgPool) -> Result<()> {
    #[derive(Eq, Hash, PartialEq, sqlx::FromRow)]
    struct ValidRoom {
        room: RoomNumber,
    }

    // Ignore pulling if VALID_ROOM_SET is set.
    if VALID_ROOM_SET.get().is_some() {
        return Ok(());
    }
    let room_list: Vec<ValidRoom> = sqlx::query_as("SELECT id AS room FROM dormitory_room;")
        .fetch_all(db)
        .await?;

    let set = HashSet::from_iter(room_list.into_iter().map(|s| s.room));
    VALID_ROOM_SET.set(set).expect("Failed to set VALID_ROOM_SET");
    Ok(())
}

fn get_valid_room_set() -> &'static HashSet<RoomNumber> {
    VALID_ROOM_SET
        .get()
        .expect("init_valid_room_set() should be called first.")
}

async fn request_room_balance() -> Result<Vec<RawBalance>> {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();

    static DATA_SOURCE_URL: &str =
        "https://xgfy.sit.edu.cn/unifri-flow/WF/Comm/ProcessRequest.do?DoType=DBAccess_RunSQLReturnTable";

    params.insert("SQL", "select * from sys_room_balance;");
    let response = client
        .post(DATA_SOURCE_URL)
        .header("Cookie", "FK_Dept=B1101")
        .form(&params)
        .send()
        .await?;

    response.json::<Vec<RawBalance>>().await.map_err(Into::into)
}

async fn get_balance_list() -> Result<Vec<RawBalance>> {
    let raw_response = request_room_balance().await?;
    let filter = |r: &RawBalance| r.room != INVALID_ROOM_NUMBER;
    let result = raw_response.into_iter().filter(filter).collect();

    Ok(result)
}

async fn update_db(db: &PgPool, records: Vec<RawBalance>) -> Result<()> {
    let current = Local::now();
    let rooms: Vec<i32> = records.iter().map(|x| x.room).collect();
    let balance: Vec<f32> = records.iter().map(|x| x.total).collect();

    // Consumption is calculated by dormitory_balance_trigger on PostgreSQL.
    // Do not delete all data before any INSERT statement.
    // Here, we use a single SQL statement instead of a for loop to speed up updating process.
    sqlx::query(
        "INSERT INTO dormitory_balance
                (room, total_balance, ts)
            SELECT *, $3::timestamptz AS ts FROM UNNEST($1::int[], $2::float[])
            ON CONFLICT (room) DO UPDATE SET total_balance = excluded.total_balance, ts = excluded.ts;",
    )
    .bind(rooms)
    .bind(balance)
    .bind(current)
    .execute(db)
    .await?;

    Ok(())
}

pub async fn pull_balance_list(db: &PgPool) -> Result<()> {
    init_valid_room_set(db).await?;

    let start = Instant::now();
    let result = get_balance_list().await?;
    tracing::info!("get {} records, cost {}s", result.len(), start.elapsed().as_secs_f32());

    let start = Instant::now();
    let count = result.len();
    update_db(db, result).await?;
    tracing::info!("save {} records, cost {}s", count, start.elapsed().as_secs_f32());

    clear_cache();
    Ok(())
}
