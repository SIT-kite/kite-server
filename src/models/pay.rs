use crate::error::{ApiError, Result};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, sqlx::FromRow)]
/// Electricity Balance for FengXian dormitory.
pub struct ElectricityBalance {
    /// Room id in the format described in the doc.
    pub room: i32,
    /// Total available amount
    pub balance: f32,
    /// Available power
    pub power: f32,
    /// Last update time
    pub ts: NaiveDateTime,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
/// Electricity usage statistics
pub struct ElectricityBill {
    /// Room Id
    pub date: String,
    /// Charge amount in estimation.
    pub charge: f32,
    /// Consumption amount in estimation.
    pub consumption: f32,
}

pub enum AggregationLevel {
    Time,
    Hour,
    Day,
    Week,
    Month,
}

pub struct BalanceManager<'a> {
    db: &'a sqlx::PgPool,
}

#[derive(thiserror::Error, Debug, ToPrimitive)]
pub enum BalanceError {
    #[error("无对应房间数据")]
    NoSuchRoom = 200,
}

impl<'a> BalanceManager<'a> {
    pub fn new(db: &'a sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn query_last_balance(self, room: i32) -> Result<ElectricityBalance> {
        let balance: Option<ElectricityBalance> = sqlx::query_as(
            "SELECT room, total_balance AS balance, CAST(total_balance / 0.6 AS real) AS power, ts
                FROM dormitory.balance WHERE room = $1 ORDER BY ts DESC LIMIT 1",
        )
        .bind(room)
        .fetch_optional(self.db)
        .await?;

        balance.ok_or(ApiError::new(BalanceError::NoSuchRoom))
    }

    pub async fn query_statistics_by_day(
        self,
        room: i32,
        start_date: String,
        end_date: String,
    ) -> Result<Vec<ElectricityBill>> {
        let bills = sqlx::query_as(
            "SELECT d.day AS date, COALESCE(records.charged_amount, 0.00) AS charge, ABS(COALESCE(records.used_amount, 0.00)) AS consumption
                FROM
                    (SELECT to_char(day_range, 'yyyy-MM-dd') AS day FROM generate_series($1::date,  $2::date, '1 day') AS day_range) d
                LEFT JOIN (
                    SELECT * FROM dormitory.get_consumption_report_by_day($1::date, CAST($2::date + '1 day'::interval AS date), $3)
                ) AS records
                ON d.day = records.day;")
            .bind(start_date)
            .bind(end_date)
            .bind(room)
            .fetch_all(self.db)
            .await?;

        Ok(bills)
    }

    // pub fn query_balance_statistics(self, room: String, level: AggregationLevel) {}
}
