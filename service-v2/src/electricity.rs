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

use std::ops::Sub;

use chrono::{Date, Local};
use chrono::{DateTime, Duration};
use poem::handler;
use poem::web::{Data, Json, Path, Query};
use serde::Serialize;
use sqlx::PgPool;

use kite::model::balance as model;

use crate::error::Result;
use crate::response::ApiResponse;

#[derive(Serialize)]
/// Electricity Balance for FengXian dormitory.
pub struct ElectricityBalance {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
    /// Remaining power level
    pub power: f32,
    /// Last update time
    pub ts: DateTime<Local>,
}

impl Into<ElectricityBalance> for model::ElectricityBalance {
    fn into(self) -> ElectricityBalance {
        let model::ElectricityBalance { room, balance, ts } = self;
        ElectricityBalance {
            room,
            balance,
            power: self.balance / 0.6,
            ts,
        }
    }
}

#[handler]
pub async fn query_room_balance(pool: Data<&PgPool>, Path(room): Path<i32>) -> Result<Json<serde_json::Value>> {
    let data: Option<ElectricityBalance> = model::get_latest_balance(&pool, room).await?.map(Into::into);

    let content: serde_json::Value = if let Some(data) = data {
        ApiResponse::normal(data).into()
    } else {
        ApiResponse::<()>::fail(404, "No such room.".to_string()).into()
    };
    Ok(Json(content))
}

#[handler]
pub async fn query_room_consumption_rank(
    pool: Data<&PgPool>,
    Path(room): Path<i32>,
) -> Result<Json<serde_json::Value>> {
    let data = model::get_consumption_rank(&pool, room).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[derive(serde::Deserialize)]
pub struct DateRange {
    start: Option<String>,
    end: Option<String>,
}

#[handler]
pub async fn query_room_bills_by_day(
    pool: Data<&PgPool>,
    Path(room): Path<i32>,
    Query(parameters): Query<DateRange>,
) -> Result<Json<serde_json::Value>> {
    let today = chrono::Local::today();
    let to_str = |x: Date<Local>| x.format("%Y-%m-%d").to_string();

    let start_date = parameters.start.unwrap_or_else(|| to_str(today.sub(Duration::days(7))));
    let end_date = parameters.end.unwrap_or_else(|| to_str(today));

    let data = model::get_bill_in_day(&pool, room, start_date, end_date).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}

#[handler]
pub async fn query_room_bills_by_hour(pool: Data<&PgPool>, Path(room): Path<i32>) -> Result<Json<serde_json::Value>> {
    let now = chrono::Local::now();

    let start_time = now.sub(Duration::days(1));
    let end_time = now;

    let data = model::get_bill_in_hour(&pool, room, start_time, end_time).await?;
    let response: serde_json::Value = ApiResponse::normal(data).into();

    Ok(Json(response))
}
