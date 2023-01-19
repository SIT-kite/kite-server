/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2021-2023 上海应用技术大学 上应小风筝团队
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

use chrono::{DateTime, Duration, Local};
use sqlx::PgPool;
use tonic::{Request, Response, Status};

use crate::error::ToStatus;
use crate::model::{balance as model, ToTimestamp};
pub use crate::service::gen::balance as gen;

impl Into<gen::RoomBalance> for model::ElectricityBalance {
    fn into(self) -> gen::RoomBalance {
        gen::RoomBalance {
            room: self.room,
            balance: self.balance,
            ts: Some(ToTimestamp::datetime(self.ts)),
        }
    }
}

impl Into<gen::BillItem> for model::DailyElectricityBill {
    fn into(self) -> gen::BillItem {
        use gen::bill_item::Identifier;

        gen::BillItem {
            increment: self.charge,
            decrement: self.consumption,
            identifier: Some(Identifier::Date(self.date)),
        }
    }
}

impl Into<gen::BillItem> for model::HourlyElectricityBill {
    fn into(self) -> gen::BillItem {
        use gen::bill_item::Identifier;

        gen::BillItem {
            increment: self.charge,
            decrement: self.consumption,
            identifier: Some(Identifier::Time(self.time)),
        }
    }
}

impl Into<gen::ConsumptionRank> for model::RecentConsumptionRank {
    fn into(self) -> gen::ConsumptionRank {
        gen::ConsumptionRank {
            consumption: self.consumption,
            rank: self.rank,
            total_room: self.room_count,
        }
    }
}

async fn get_latest_balance(pool: &PgPool, room: i32) -> Result<model::ElectricityBalance, tonic::Status> {
    sqlx::query_as(
        "SELECT room, total_balance AS balance, ts
             FROM dormitory.balance
             WHERE room = $1
             ORDER BY ts DESC
             LIMIT 1",
    )
    .bind(room)
    .fetch_optional(pool)
    .await
    .map_err(ToStatus::to_status)?
    .ok_or_else(|| tonic::Status::not_found("No such room"))
}

async fn get_bill_in_day(
    pool: &PgPool,
    room: i32,
    from: String,
    to: String,
) -> Result<Vec<model::DailyElectricityBill>, tonic::Status> {
    sqlx::query_as(
        "SELECT d.day AS date, COALESCE(records.charged_amount, 0.00) AS charge, ABS(COALESCE(records.used_amount, 0.00)) AS consumption
                FROM (SELECT to_char(day_range, 'yyyy-MM-dd') AS day FROM generate_series($1::date,  $2::date, '1 day') AS day_range) d
                LEFT JOIN (SELECT * FROM dormitory.get_consumption_report_by_day($1::date, CAST($2::date + '1 day'::interval AS date), $3)) AS records
                ON d.day = records.day;")
        .bind(from)
        .bind(to)
        .bind(room)
        .fetch_all(pool)
        .await
        .map_err(ToStatus::to_status)
}

async fn get_bill_in_hour(
    pool: &PgPool,
    room: i32,
    from: DateTime<Local>,
    to: DateTime<Local>,
) -> Result<Vec<model::HourlyElectricityBill>, tonic::Status> {
    sqlx::query_as(
        "SELECT h.hour AS time, COALESCE(records.charged_amount, 0.00) AS charge, ABS(COALESCE(records.used_amount, 0.00)) AS consumption
                FROM (
                    SELECT to_char(hour_range, 'yyyy-MM-dd HH24:00') AS hour
                    FROM generate_series($1::timestamptz, $2::timestamptz, '1 hour') AS hour_range) h
                LEFT JOIN (
                    SELECT * FROM dormitory.get_consumption_report_by_hour($1::timestamptz, $2::timestamptz, $3)) AS records
                ON h.hour = records.hour;")
        .bind(from)
        .bind(to)
        .bind(room)
        .fetch_all(pool)
        .await
        .map_err(ToStatus::to_status)
}

async fn get_consumption_rank(pool: &PgPool, room: i32) -> Result<model::RecentConsumptionRank, tonic::Status> {
    // TODO: Use proc_macro to reduce boilerplate code
    if let Ok(Some(cache)) = kite::cache_query!(Duration::hours(1), room) {
        return Ok(cache);
    }
    let result: model::RecentConsumptionRank =
        sqlx::query_as("SELECT room, consumption, rank, room_count FROM dormitory.get_room_24hour_rank($1);")
            .bind(room)
            .fetch_optional(pool)
            .await
            .map_err(ToStatus::to_status)?
            .ok_or_else(|| tonic::Status::not_found("No such room"))?;

    kite::cache_save!(result.clone(), room);
    Ok(result)
}

#[tonic::async_trait]
impl gen::balance_service_server::BalanceService for super::KiteGrpcServer {
    async fn get_room_balance(
        &self,
        request: Request<gen::BalanceRequest>,
    ) -> Result<Response<gen::RoomBalance>, Status> {
        let room = request.into_inner().room_number;
        let response = get_latest_balance(&self.db, room).await.map(Into::into)?;

        Ok(Response::new(response))
    }

    async fn get_consumption_rank(
        &self,
        request: Request<gen::BalanceRequest>,
    ) -> Result<Response<gen::ConsumptionRank>, Status> {
        let room = request.into_inner().room_number;
        let response = get_consumption_rank(&self.db, room).await.map(Into::into)?;

        Ok(Response::new(response))
    }

    async fn get_bill(&self, request: Request<gen::BillRequest>) -> Result<Response<gen::BillResponse>, Status> {
        let request = request.into_inner();

        // TODO:
        // It's defined that the value of gen::BillType::Daily is 0 and that of gen::BillType::Hourly is 1.
        // However, according to the reason I don't know till now, tonic (with the compiler prost) sees the "request.type" as i32,
        // So I can't use enum match  :-(
        const IS_DAILY: i32 = 0;
        const IS_HOURLY: i32 = 1;

        let bill_list: Vec<gen::BillItem> = match request.r#type {
            IS_DAILY => {
                let to_str = |x: DateTime<Local>| x.format("%Y-%m-%d").to_string();

                let today = Local::now();
                let last_week = today.sub(Duration::days(7));

                get_bill_in_day(&self.db, request.room_number, to_str(last_week), to_str(today))
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect()
            }
            IS_HOURLY => {
                let now = Local::now();
                let yesterday = now.sub(Duration::hours(24));

                get_bill_in_hour(&self.db, request.room_number, yesterday, now)
                    .await?
                    .into_iter()
                    .map(Into::into)
                    .collect()
            }
            _ => {
                return Err(tonic::Status::invalid_argument("Bill type is unexpected"));
            }
        };
        Ok(Response::new(gen::BillResponse { bill_list }))
    }
}
