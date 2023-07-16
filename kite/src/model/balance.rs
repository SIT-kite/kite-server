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
use bincode::{Decode, Encode};
use chrono::{DateTime, Local};
use serde::Serialize;
use sqlx::{FromRow, PgPool};

use crate as kite;

#[derive(Clone, Encode, Decode, FromRow)]
/// Electricity Balance for FengXian dormitory.
pub struct ElectricityBalance {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
    /// Last update time
    #[bincode(with_serde)]
    pub ts: DateTime<Local>,
}

/// Electricity usage statistics by day
#[derive(Clone, Encode, Decode, Serialize, FromRow)]
pub struct DailyElectricityBill {
    /// Date string in 'yyyy-mm-dd'
    pub date: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

/// Electricity usage statistics by hour
#[derive(Clone, Encode, Decode, Serialize, FromRow)]
pub struct HourlyElectricityBill {
    /// Hour string in 'yyyy-mm-dd HH24:00'
    pub time: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

/// Rank of recent-24hour consumption
#[derive(Clone, Encode, Decode, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct RecentConsumptionRank {
    /// Consumption in last 24 hours.
    pub consumption: f32,
    /// Rank
    pub rank: i32,
    /// Total room count
    pub room_count: i32,
}

#[crate::cache_result(timeout = 900)]
pub async fn get_latest_balance(pool: &PgPool, room: i32) -> Result<Option<ElectricityBalance>> {
    sqlx::query_as(
        "SELECT room, total_balance AS balance, ts
             FROM dormitory_balance
             WHERE room = $1
             ORDER BY ts DESC
             LIMIT 1",
    )
    .bind(room)
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

#[crate::cache_result(timeout = 43200)]
pub async fn get_bill_in_day(pool: &PgPool, room: i32, from: String, to: String) -> Result<Vec<DailyElectricityBill>> {
    sqlx::query_as(
        "SELECT d.day AS date, COALESCE(records.charged_amount, 0.00) AS charge, ABS(COALESCE(records.used_amount, 0.00)) AS consumption
                FROM (SELECT to_char(day_range, 'yyyy-MM-dd') AS day FROM generate_series($1::date,  $2::date, '1 day') AS day_range) d
                LEFT JOIN (SELECT * FROM dormitory_consumption_get_report_by_day($1::date, CAST($2::date + '1 day'::interval AS date), $3)) AS records
                ON d.day = records.day;")
        .bind(&from)
        .bind(&to)
        .bind(room)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

#[crate::cache_result(timeout = 3600)]
pub async fn get_bill_in_hour(
    pool: &PgPool,
    room: i32,
    from: DateTime<Local>,
    to: DateTime<Local>,
) -> Result<Vec<HourlyElectricityBill>> {
    sqlx::query_as(
        "SELECT h.hour AS time, COALESCE(records.charged_amount, 0.00) AS charge, ABS(COALESCE(records.used_amount, 0.00)) AS consumption
                FROM (
                    SELECT to_char(hour_range, 'yyyy-MM-dd HH24:00') AS hour
                    FROM generate_series($1::timestamptz, $2::timestamptz, '1 hour') AS hour_range) h
                LEFT JOIN (
                    SELECT * FROM dormitory_consumption_get_report_by_hour($1::timestamptz, $2::timestamptz, $3)) AS records
                ON h.hour = records.hour;")
        .bind(from)
        .bind(to)
        .bind(room)
        .fetch_all(pool)
        .await
        .map_err(Into::into)
}

#[crate::cache_result(timeout = 3600)]
pub async fn get_consumption_rank(pool: &PgPool, room: i32) -> Result<Option<RecentConsumptionRank>> {
    // The value of 'SELECT COUNT(*) FROM dormitory_room;' is 4565, which will not change in a long future.
    // And be careful, room_count is of i32, while COUNT(*) returns a long long (int8) type.
    sqlx::query_as("SELECT room, consumption, rank, 4565 AS room_count FROM dormitory_consumption_ranking
                    WHERE room = $1;")
        .bind(room)
        .fetch_optional(pool)
        .await
        .map_err(Into::into)
}
