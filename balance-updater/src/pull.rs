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
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, PgPool};
use tokio::time::Instant;

use kite::model::balance as model;
use kite::model::balance::ElectricityBalance;

#[derive(Serialize, Deserialize)]
struct RawBalance {
    #[serde(rename = "RoomName")]
    room: String,
    #[serde(rename = "Balance")]
    total: String,
}

type RoomNumber = i32;

#[derive(Eq, Hash, PartialEq, sqlx::FromRow)]
struct ValidRoom {
    room: RoomNumber,
}

async fn get_valid_room(db: &PgPool) -> Result<Vec<ValidRoom>> {
    sqlx::query_as("SELECT id AS room FROM dormitory_room;")
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

fn filter_valid_room(set: &HashSet<RoomNumber>, room: &str) -> Option<i32> {
    let room_number = room.parse::<i32>();

    if let Ok(room_number) = room_number {
        if set.contains(&room_number) {
            return Some(room_number);
        }
    }
    None
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

async fn get_balance_list(room_set: &HashSet<RoomNumber>) -> Result<Vec<model::ElectricityBalance>> {
    let current = Local::now();

    let raw_response = request_room_balance().await?;
    let result = raw_response
        .into_iter()
        .filter_map(|raw| {
            filter_valid_room(room_set, &raw.room).map(|id| model::ElectricityBalance {
                room: id,
                balance: raw.total.parse().unwrap_or_default(),
                ts: current.clone(),
            })
        })
        .collect();

    Ok(result)
}

async fn update_db(db: &PgPool, records: Vec<ElectricityBalance>) -> Result<()> {
    let rooms: Vec<i32> = records.iter().map(|x| x.room).collect();
    let balance: Vec<f32> = records.iter().map(|x| x.balance).collect();
    let ts = records.first().map(|x| x.ts);

    if let Some(ts) = ts {
        let mut tx = db.begin().await?;

        sqlx::query("DELETE FROM dormitory_balance;").execute(&mut tx).await?;
        sqlx::query(
            "INSERT INTO dormitory_balance
                (room, total_balance, ts)
            SELECT *, $3::timestamptz AS ts FROM UNNEST($1::int[], $2::float[]);",
        )
        .bind(rooms)
        .bind(balance)
        .bind(ts)
        .execute(&mut tx)
        .await?;

        tx.commit().await?;
    }
    Ok(())
}

pub async fn pull_balance_list(db: &PgPool) -> Result<()> {
    let valid_rooms = get_valid_room(&db).await?;
    let room_set = HashSet::from_iter(valid_rooms.into_iter().map(|s| s.room));

    let start = Instant::now();
    let result = get_balance_list(&room_set).await?;
    tracing::info!("get {} records, cost {}s", result.len(), start.elapsed().as_secs_f32());

    let start = Instant::now();
    let count = result.len();
    update_db(db, result).await?;
    tracing::info!("save {} records, cost {}s", count, start.elapsed().as_secs_f32());

    Ok(())
}
